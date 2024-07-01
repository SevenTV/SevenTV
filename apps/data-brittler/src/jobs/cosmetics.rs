use std::collections::HashSet;
use std::sync::Arc;
use std::vec;

use bson::oid::ObjectId;
use mongodb::bson::doc;
use shared::database::badge::Badge;
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::paint::{Paint, PaintLayer, PaintLayerId, PaintLayerType};
use shared::database::{self, Collection};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{download_cosmetics, error, types};

pub struct CosmeticsJob {
	global: Arc<Global>,
	all_tasks: HashSet<String>,
}

impl CosmeticsJob {
	async fn request_image(&self, cosmetic_id: ObjectId, url: &str) -> Result<bytes::Bytes, ProcessOutcome> {
		download_cosmetics::request_image(&self.global, url)
			.await
			.map_err(|e| match e {
				download_cosmetics::RequestImageError::Reqwest(e) => ProcessOutcome::error(e),
				download_cosmetics::RequestImageError::Status(status) => {
					ProcessOutcome::error(error::Error::ImageDownload { cosmetic_id, status })
				}
			})
	}
}

impl Job for CosmeticsJob {
	type T = types::Cosmetic;

	const NAME: &'static str = "transfer_cosmetics";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping paints and badges collections");
			Paint::collection(global.target_db()).delete_many(doc! {}).await?;
			Badge::collection(global.target_db()).delete_many(doc! {}).await?;
		}

		Ok(Self {
			global,
			all_tasks: HashSet::new(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("cosmetics")
	}

	async fn process(&mut self, cosmetic: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let ip = self.global.image_processor();

		match cosmetic.data {
			types::CosmeticData::Badge { tooltip, tag } => {
				let id = cosmetic.id.into();

				let download_url = format!("https://cdn.7tv.app/badge/{}/2x", cosmetic.id);
				let image_data = match self.request_image(cosmetic.id, &download_url).await {
					Ok(data) => data,
					Err(outcome) => return outcome,
				};

				let input = match ip.upload_badge(id, image_data).await {
					Ok(scuffle_image_processor_proto::ProcessImageResponse { error: Some(error), .. }) => {
						return outcome.with_error(error::Error::ImageProcessor(error));
					}
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
						self.all_tasks.insert(id.clone());
						ImageSetInput::Pending {
							task_id: id,
							path: path.path,
							mime: content_type,
							size: size,
						}
					}
					Err(e) => return outcome.with_error(e),
					_ => return outcome.with_error(error::Error::NotImplemented("missing image upload info")),
				};

				let image_set = ImageSet {
					input,
					..Default::default()
				};

				let tags = tag.map(|t| vec![t]).unwrap_or_default();
				match Badge::collection(self.global.target_db())
					.insert_one(Badge {
						id: cosmetic.id.into(),
						name: cosmetic.name,
						description: tooltip,
						tags,
						image_set,
					})
					.await
				{
					Ok(_) => outcome.inserted_rows += 1,
					Err(e) => outcome.errors.push(e.into()),
				}
			}
			types::CosmeticData::Paint { data, drop_shadows } => {
				let id = cosmetic.id.into();

				let layer_id = PaintLayerId::new();

				let layer = match data {
					types::PaintData::LinearGradient {
						stops, repeat, angle, ..
					} => Some(PaintLayerType::LinearGradient {
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
					} => Some(PaintLayerType::RadialGradient {
						angle,
						repeating: repeat,
						stops: stops.into_iter().map(Into::into).collect(),
						shape,
					}),
					types::PaintData::Url {
						image_url: Some(image_url),
						..
					} => {
						let image_data = match self.request_image(cosmetic.id, &image_url).await {
							Ok(data) => data,
							Err(outcome) => return outcome,
						};

						let input = match ip.upload_paint_layer(id, layer_id, image_data).await {
							Ok(scuffle_image_processor_proto::ProcessImageResponse { error: Some(error), .. }) => {
								return outcome.with_error(error::Error::ImageProcessor(error));
							}
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
								self.all_tasks.insert(id.clone());
								ImageSetInput::Pending {
									task_id: id,
									path: path.path,
									mime: content_type,
									size: size,
								}
							}
							Err(e) => return outcome.with_error(e),
							_ => return outcome.with_error(error::Error::NotImplemented("missing image upload info")),
						};

						Some(PaintLayerType::Image(ImageSet {
							input,
							..Default::default()
						}))
					}
					types::PaintData::Url { image_url: None, .. } => None,
				};

				let paint_data = database::paint::PaintData {
					layers: layer
						.map(|ty| {
							vec![PaintLayer {
								id: layer_id,
								ty,
								..Default::default()
							}]
						})
						.unwrap_or_default(),
					shadows: drop_shadows.into_iter().map(Into::into).collect(),
				};

				match Paint::collection(self.global.target_db())
					.insert_one(Paint {
						id,
						name: cosmetic.name,
						description: String::new(),
						tags: vec![],
						data: paint_data,
					})
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

	async fn finish(self) -> ProcessOutcome {
		match self.global.all_tasks().set(self.all_tasks) {
			Ok(_) => ProcessOutcome::default(),
			Err(e) => ProcessOutcome {
				errors: vec![e.into()],
				..Default::default()
			},
		}
	}
}
