use std::sync::Arc;
use std::vec;

use mongodb::bson::doc;
use shared::database::{self, Badge, Collection, FileSet, FileSetId, FileSetKind, FileSetProperties, Paint};
use scuffle_image_processor_proto::{input, DrivePath, Events, Input, Output, ProcessImageRequest, Task};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

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

			tracing::info!("deleting all paint and badge file sets");
			FileSet::collection(global.target_db())
				.delete_many(
					doc! {
						"kind": {
							"$in": mongodb::bson::to_bson(&[
								FileSetKind::Paint,
								FileSetKind::Badge,
							])?
						}
					},
					None,
				)
				.await?;
		}

		Ok(Self {
			global,
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("cosmetics")
	}

	async fn process(&mut self, cosmetic: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		match cosmetic.data {
			types::CosmeticData::Badge { tooltip, tag } => {
				let file_set_id = FileSetId::with_timestamp(cosmetic.id.timestamp().to_chrono());

				// TODO: image file set properties
				// TODO: maybe also reupload the image to the image processor because it's only
				// available in webp right now
				let properties = FileSetProperties::Other(shared::database::FileProperties {
					path: format!("cdn.7tv.app/badge/{}/1x", cosmetic.id),
					size: 0,
					mime: Some("image/webp".to_string()),
					extra: (),
				});

				match FileSet::collection(self.global.target_db())
					.insert_one(
						FileSet {
							id: file_set_id,
							kind: FileSetKind::Badge,
							authenticated: false,
							properties,
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

				let tags = tag.map(|t| vec![t]).unwrap_or_default();
				match Badge::collection(self.global.target_db())
					.insert_one(
						Badge {
							id: cosmetic.id.into(),
							name: cosmetic.name,
							description: tooltip,
							tags,
							file_set_id,
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
				let (layer, file_set_ids) = match data {
					types::PaintData::LinearGradient {
						stops, repeat, angle, ..
					} => (
						Some(database::PaintLayerType::LinearGradient {
							angle,
							repeating: repeat,
							stops: stops.into_iter().map(Into::into).collect(),
						}),
						vec![],
					),
					types::PaintData::RadialGradient {
						stops,
						repeat,
						angle,
						shape,
						..
					} => (
						Some(database::PaintLayerType::RadialGradient {
							angle,
							repeating: repeat,
							stops: stops.into_iter().map(Into::into).collect(),
							shape,
						}),
						vec![],
					),
					types::PaintData::Url {
						image_url: Some(image_url),
						..
					} => {
						let file_set_id = FileSetId::with_timestamp(cosmetic.id.timestamp().to_chrono());

						let processor_request = ProcessImageRequest {
							task: Some(Task {
								input: Some(Input {
									path: Some(input::Path::PublicUrl(image_url)),
									..Default::default()
								}),
								output: Some(Output {
									drive_path: Some(DrivePath {
										drive: "public_s3".to_string(),
										path: format!("paint/{}", file_set_id),
									}),
									..Default::default()
								}),
								events: Some(Events {
									..Default::default()
								}),
								limits: None,
							}),
							..Default::default()
						};

						// TODO: upload image data to s3 input bucket
						let properties = FileSetProperties::Image {
							input: todo!(),
							pending: true,
							outputs: vec![],
						};

						match FileSet::collection(self.global.target_db())
							.insert_one(
								FileSet {
									id: file_set_id,
									kind: FileSetKind::Paint,
									authenticated: false,
									properties,
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

						(Some(database::PaintLayerType::Image(file_set_id)), vec![file_set_id])
					}
					types::PaintData::Url { image_url: None, .. } => (None, vec![]),
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
							file_set_ids,
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
