use std::{pin::Pin, sync::Arc, vec};

use postgres_types::Type;
use shared::database::{self, FileSetKind, FileSetProperties};
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use crate::{database::file_sets_writer, error, global::Global, types};

use super::{Job, ProcessOutcome};

pub struct CosmeticsJob {
	global: Arc<Global>,
	http_client: reqwest::Client,
	file_sets_writer: Pin<Box<BinaryCopyInWriter>>,
	paints_writer: Pin<Box<BinaryCopyInWriter>>,
	paint_file_sets_writer: Pin<Box<BinaryCopyInWriter>>,
	badges_writer: Pin<Box<BinaryCopyInWriter>>,
}

impl CosmeticsJob {
	pub async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating paints, paint_file_sets and badges table");
			scuffle_utils::database::query("TRUNCATE paints, paint_file_sets, badges")
				.build()
				.execute(global.db())
				.await?;

			tracing::info!("deleting all paint and badge file sets");
			scuffle_utils::database::query("DELETE FROM file_sets WHERE kind = 'PAINT' OR kind = 'BADGE'")
				.build()
				.execute(global.db())
				.await?;
		}

		let file_sets_writer = file_sets_writer(&global).await?;

		let paints_client = global.db().get().await?;
		let paints_writer = BinaryCopyInWriter::new(
			paints_client
				.copy_in("COPY paints (id, name, data) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::VARCHAR, Type::JSONB],
		);

		let paint_file_sets_client = global.db().get().await?;
		let paint_file_sets_writer = BinaryCopyInWriter::new(
			paint_file_sets_client
				.copy_in("COPY paint_file_sets (paint_id, file_set_id) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::UUID],
		);

		let badges_client = global.db().get().await?;
		let badges_writer = BinaryCopyInWriter::new(
			badges_client
				.copy_in("COPY badges (id, name, description, tags, file_set_id) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::VARCHAR, Type::TEXT, Type::TEXT_ARRAY, Type::UUID],
		);

		Ok(Self {
			global,
			http_client: reqwest::Client::new(),
			file_sets_writer: Box::pin(file_sets_writer),
			paints_writer: Box::pin(paints_writer),
			paint_file_sets_writer: Box::pin(paint_file_sets_writer),
			badges_writer: Box::pin(badges_writer),
		})
	}
}

impl Job for CosmeticsJob {
	type T = types::Cosmetic;

	const NAME: &'static str = "transfer_cosmetics";

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("cosmetics")
	}

	async fn process(&mut self, cosmetic: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = cosmetic.id.into_ulid();

		match cosmetic.data {
			types::CosmeticData::Badge { tooltip, tag } => {
				let file_set_id = ulid::Ulid::from_datetime(id.datetime());

				// TODO: image file set properties
				let properties = FileSetProperties::Other(shared::database::FileProperties {
					path: format!("cdn.7tv.app/badge/{}/1x", cosmetic.id),
					size: 0,
					mime: Some("image/webp".to_string()),
					extra: (),
				});

				match self
					.file_sets_writer
					.as_mut()
					.write(&[&file_set_id, &FileSetKind::Badge, &false, &postgres_types::Json(properties)])
					.await
				{
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => {
						outcome.errors.push(e.into());
						return outcome;
					}
				}

				let tags = tag.map(|t| vec![t]).unwrap_or_default();
				match self
					.badges_writer
					.as_mut()
					.write(&[&id, &cosmetic.name, &tooltip, &tags, &file_set_id])
					.await
				{
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
			types::CosmeticData::Paint { data, drop_shadows } => {
				let layer = match data {
					types::PaintData::LinearGradient {
						stops, repeat, angle, ..
					} => Some(database::PaintLayerType::LinearGradient {
						angle,
						repeating: repeat,
						stops: stops.into_iter().map(Into::into).collect(),
					}),
					types::PaintData::RadialGradient {
						stops,
						repeat,
						angle,
						shape,
						..
					} => Some(database::PaintLayerType::RadialGradient {
						angle,
						repeating: repeat,
						stops: stops.into_iter().map(Into::into).collect(),
						shape,
					}),
					types::PaintData::Url {
						image_url: Some(image_url),
						..
					} => {
						let file_set_id = ulid::Ulid::from_datetime(id.datetime());

						let image_data = match self.http_client.get(image_url).send().await {
							Ok(res) => res.bytes().await.unwrap(),
							Err(e) => {
								outcome.errors.push(error::Error::PaintImageUrlRequest(e));
								return outcome;
							}
						};
						// TODO: upload image data to s3 input bucket
						let properties = FileSetProperties::Image {
							input: todo!(),
							pending: true,
							outputs: vec![],
						};

						match self
							.file_sets_writer
							.as_mut()
							.write(&[&file_set_id, &FileSetKind::Paint, &false, &postgres_types::Json(properties)])
							.await
						{
							Ok(_) => outcome.inserted_rows += 1,
							Err(e) => {
								outcome.errors.push(e.into());
								return outcome;
							}
						}

						match self.paint_file_sets_writer.as_mut().write(&[&id, &file_set_id]).await {
							Ok(_) => outcome.inserted_rows += 1,
							Err(e) => {
								outcome.errors.push(e.into());
								return outcome;
							}
						}

						Some(database::PaintLayerType::Image(file_set_id))
					}
					types::PaintData::Url { image_url: None, .. } => None,
				};

				let paint_data = database::PaintData {
					layers: layer
						.map(|ty| vec![database::PaintLayer { ty, opacity: 1.0 }])
						.unwrap_or_default(),
					shadows: drop_shadows.into_iter().map(Into::into).collect(),
				};

				match self
					.paints_writer
					.as_mut()
					.write(&[&id, &cosmetic.name, &postgres_types::Json(paint_data)])
					.await
				{
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => {
						outcome.errors.push(e.into());
						return outcome;
					}
				}
			}
		}

		outcome
	}
}
