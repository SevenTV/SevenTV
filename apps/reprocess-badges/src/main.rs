use scuffle_foundations::bootstrap::{bootstrap, Bootstrap};
use scuffle_foundations::settings::auto_settings;
use scuffle_foundations::settings::cli::Matches;
use shared::config::{DatabaseConfig, ImageProcessorConfig};
use shared::database::badge::Badge;
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::queries::{filter, update};
use shared::database::MongoCollection;
use shared::image_processor::ImageProcessor;

mod badges;

#[auto_settings]
#[serde(default)]
struct Config {
	database: DatabaseConfig,
	image_processor: ImageProcessorConfig,
}

impl Bootstrap for Config {
	type Settings = Self;
}

#[bootstrap]
async fn main(settings: Matches<Config>) {
	let mongo = shared::database::setup_database(&settings.settings.database, false)
		.await
		.unwrap();
	let db = mongo.default_database().unwrap();

	let ip = ImageProcessor::new(&settings.settings.image_processor)
		.await
		.expect("failed to initialize image processor");

	for job in badges::jobs() {
		tracing::info!("reprocessing {:?}", job.input);

		let data = tokio::fs::read(job.input).await.expect("failed to read input file");

		// The api should be running and will take care of the image processor callback
		match ip.upload_badge(job.id, data.into()).await {
			Ok(scuffle_image_processor_proto::ProcessImageResponse {
				id,
				upload_info:
					Some(scuffle_image_processor_proto::ProcessImageResponseUploadInfo {
						path: Some(path),
						content_type,
						size,
					}),
				error: None,
			}) => {
				let image_set = ImageSet {
					input: ImageSetInput::Pending {
						task_id: id,
						path: path.path,
						mime: content_type,
						size: size as i64,
					},
					outputs: vec![],
				};

				Badge::collection(&db)
					.update_one(
						filter::filter! {
							Badge {
								#[query(rename = "_id")]
								id: job.id,
							}
						},
						update::update! {
							#[query(set)]
							Badge {
								#[query(serde)]
								image_set,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
					)
					.await
					.expect("failed to clear image set");
			}
			Ok(res) => {
				tracing::error!(res = ?res, "invalid image processor response");
				continue;
			}
			Err(e) => {
				tracing::error!(error = ?e, "failed to start send image processor request");
				continue;
			}
		}
	}

	std::process::exit(0);
}
