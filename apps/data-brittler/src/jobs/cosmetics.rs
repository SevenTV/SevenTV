use std::sync::Arc;
use std::vec;

use mongodb::bson::doc;
use scuffle_image_processor_proto::{input, DrivePath, Events, Input, Output, ProcessImageRequest, Task};
use shared::database::{self, Badge, Collection, Id, Image, ImageSet, Paint};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct CosmeticsJob {
	global: Arc<Global>,
}

impl Job for CosmeticsJob {
	type T = types::Cosmetic;

	const NAME: &'static str = "transfer_cosmetics";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping paints and badges collections");
			Paint::collection(global.target_db()).drop(None).await?;
			Badge::collection(global.target_db()).drop(None).await?;
		}

		Ok(Self { global })
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("cosmetics")
	}

	async fn process(&mut self, cosmetic: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		match cosmetic.data {
			types::CosmeticData::Badge { tooltip, tag } => {
				// TODO: image file set properties
				// TODO: maybe also reupload the image to the image processor because it's only
				// available in webp right now
				let image_set = ImageSet {
					outputs: vec![Image {
						path: format!("badge/{}/1x", cosmetic.id),
						..Default::default()
					}],
					..Default::default()
				};

				let tags = tag.map(|t| vec![t]).unwrap_or_default();
				match Badge::collection(self.global.target_db())
					.insert_one(
						Badge {
							id: cosmetic.id.into(),
							name: cosmetic.name,
							description: tooltip,
							tags,
							image_set,
						},
						None,
					)
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
						let processor_request = ProcessImageRequest {
							task: Some(Task {
								input: Some(Input {
									path: Some(input::Path::PublicUrl(image_url)),
									..Default::default()
								}),
								output: Some(Output {
									drive_path: Some(todo!()),
									..Default::default()
								}),
								events: Some(Events { ..Default::default() }),
								limits: None,
							}),
							..Default::default()
						};

						// TODO: upload image data to s3 input bucket

						Some(database::PaintLayerType::Image(ImageSet {
							outputs: vec![Image {
								path: todo!(),
								..Default::default()
							}],
							..Default::default()
						}))
					}
					types::PaintData::Url { image_url: None, .. } => None,
				};

				let paint_data = database::PaintData {
					layers: layer
						.map(|ty| vec![database::PaintLayer { ty, opacity: 1.0 }])
						.unwrap_or_default(),
					shadows: drop_shadows.into_iter().map(Into::into).collect(),
				};

				match Paint::collection(self.global.target_db())
					.insert_one(
						Paint {
							id: cosmetic.id.into(),
							name: cosmetic.name,
							description: String::new(),
							tags: vec![],
							data: paint_data,
						},
						None,
					)
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
