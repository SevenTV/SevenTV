use std::collections::HashMap;
use std::sync::Arc;
use std::{io, vec};

use anyhow::Context;
use bson::oid::ObjectId;
use futures::StreamExt;
use scuffle_image_processor_proto::EventCallback;
use shared::database;
use shared::database::badge::{Badge, BadgeId};
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::paint::{Paint, PaintId, PaintLayer, PaintLayerId, PaintLayerType};
use shared::database::user::profile_picture::UserProfilePictureId;
use shared::database::user::UserId;
use tokio::sync::mpsc;

use super::{JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::{download_cosmetics, error, types};

async fn request_image(global: &Arc<Global>, cosmetic_id: ObjectId, url: &str) -> Result<bytes::Bytes, ProcessOutcome> {
	download_cosmetics::request_image(global, url).await.map_err(|e| match e {
		download_cosmetics::RequestImageError::Reqwest(e) => ProcessOutcome::error(e),
		download_cosmetics::RequestImageError::Status(status) => {
			ProcessOutcome::error(error::Error::ImageDownload { cosmetic_id, status })
		}
	})
}

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub badges: &'a mut HashMap<BadgeId, Badge>,
	pub paints: &'a mut HashMap<PaintId, Paint>,
	pub pending_tasks: &'a mut Vec<(PendingTask, mpsc::Receiver<EventCallback>)>,
}

#[tracing::instrument(skip_all, name = "cosmetics")]
pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("cosmetics");

	let RunInput {
		global,
		badges,
		paints,
		pending_tasks,
	} = input;

	let mut cursor = global
		.main_source_db
		.collection::<types::Cosmetic>("cosmetics")
		.find(bson::doc! {})
		.await
		.context("query")?;

	while let Some(cosmetic) = cursor.next().await {
		match cosmetic {
			Ok(cosmetic) => {
				outcome += process(ProcessInput {
					global,
					pending_tasks,
					badges,
					paints,
					cosmetic,
				})
				.await;
				outcome.processed_documents += 1;
			}
			Err(e) => {
				tracing::error!("{:#}", e);
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

#[derive(Debug)]
pub enum PendingTask {
	Badge(BadgeId),
	Paint(PaintId, PaintLayerId),
	UserProfilePicture(UserProfilePictureId),
}

struct ProcessInput<'a> {
	global: &'a Arc<Global>,
	pending_tasks: &'a mut Vec<(PendingTask, tokio::sync::mpsc::Receiver<EventCallback>)>,
	badges: &'a mut HashMap<BadgeId, Badge>,
	paints: &'a mut HashMap<PaintId, Paint>,
	cosmetic: types::Cosmetic,
}

async fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let outcome = ProcessOutcome::default();

	let ProcessInput {
		global,
		cosmetic,
		pending_tasks,
		badges,
		paints,
	} = input;

	let ip = &global.image_processor;

	match cosmetic.data {
		types::CosmeticData::Badge { tooltip, tag } => {
			let badge_id = cosmetic.id.into();

			let image_set = if global.config.should_run_cosmetics() {
				let image_data = match tokio::fs::read(format!("local/cosmetics/{}", cosmetic.id)).await {
					Ok(data) => bytes::Bytes::from(data),
					Err(e) => {
						if let io::ErrorKind::NotFound = e.kind() {
							let download_url = format!("https://cdn.7tv.app/badge/{}/3x", cosmetic.id);
							match request_image(global, cosmetic.id, &download_url).await {
								Ok(data) => data,
								Err(outcome) => return outcome,
							}
						} else {
							return outcome.with_error(e);
						}
					}
				};

				let input = match ip.upload_badge(badge_id, image_data).await {
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
						let (tx, rx) = tokio::sync::mpsc::channel(10);
						pending_tasks.push((PendingTask::Badge(badge_id), rx));
						global.all_tasks.lock().await.insert(id.clone(), tx);
						tracing::info!(task_id = %id, "started send image processor request");
						ImageSetInput::Pending {
							task_id: id,
							path: path.path,
							mime: content_type,
							size: size as i64,
						}
					}
					Err(e) => return outcome.with_error(e),
					_ => {
						return outcome.with_error(error::Error::NotImplemented("missing image upload info"));
					}
				};

				ImageSet { input, outputs: vec![] }
			} else {
				ImageSet {
					input: ImageSetInput::Pending {
						mime: "image/png".to_string(),
						path: format!("badge/{}", cosmetic.id),
						size: 0,
						task_id: "0".to_string(),
					},
					outputs: vec![],
				}
			};

			let tags = tag.map(|t| vec![t]).unwrap_or_default();
			badges.insert(
				badge_id,
				Badge {
					id: cosmetic.id.into(),
					name: cosmetic.name,
					description: Some(tooltip),
					tags,
					image_set,
					updated_at: chrono::Utc::now(),
					created_by_id: UserId::nil(),
					search_updated_at: None,
				},
			);
		}
		types::CosmeticData::Paint { data, drop_shadows } => {
			let paint_id = cosmetic.id.into();

			let layer_id = PaintLayerId::new();

			let layer = if global.config.should_run_cosmetics() {
				match data {
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
						let image_data = match tokio::fs::read(format!("local/cosmetics/{}", cosmetic.id)).await {
							Ok(data) => bytes::Bytes::from(data),
							Err(e) => {
								if let io::ErrorKind::NotFound = e.kind() {
									match request_image(global, cosmetic.id, &image_url).await {
										Ok(data) => data,
										Err(outcome) => return outcome,
									}
								} else {
									return outcome.with_error(e);
								}
							}
						};

						let input = match ip.upload_paint_layer(paint_id, layer_id, image_data).await {
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
								let (tx, rx) = tokio::sync::mpsc::channel(10);
								pending_tasks.push((PendingTask::Paint(paint_id, layer_id), rx));
								global.all_tasks.lock().await.insert(id.clone(), tx);
								tracing::info!(task_id = %id, "started send image processor request");
								ImageSetInput::Pending {
									task_id: id,
									path: path.path,
									mime: content_type,
									size: size as i64,
								}
							}
							Err(e) => return outcome.with_error(e),
							_ => return outcome.with_error(error::Error::NotImplemented("missing image upload info")),
						};

						Some(PaintLayerType::Image(ImageSet { input, outputs: vec![] }))
					}
					types::PaintData::Url { image_url: None, .. } => None,
				}
			} else {
				None
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

			paints.insert(
				paint_id,
				Paint {
					id: paint_id,
					name: cosmetic.name,
					description: None,
					tags: vec![],
					data: paint_data,
					created_by: UserId::nil(),
					search_updated_at: None,
					updated_at: chrono::Utc::now(),
				},
			);
		}
	}

	outcome
}
