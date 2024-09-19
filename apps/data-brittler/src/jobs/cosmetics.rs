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
use shared::database::user::UserId;
use tokio::sync::mpsc;

use super::{CdnFileRename, JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::{download_cosmetics, error, types};

async fn request_image(global: &Arc<Global>, cosmetic_id: ObjectId, url: &str) -> Result<bytes::Bytes, ProcessOutcome> {
	download_cosmetics::request_image(&global, url).await.map_err(|e| match e {
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
	pub cdn_rename: &'a mut Vec<CdnFileRename>,
	pub pending_tasks: &'a mut Vec<(PendingTask, mpsc::Receiver<EventCallback>)>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("cosmetics");

	let RunInput {
		global,
		badges,
		paints,
		pending_tasks,
		cdn_rename,
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
					cdn_rename,
				})
				.await;
				outcome.processed_documents += 1;
			}
			Err(e) => {
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
}

struct ProcessInput<'a> {
	global: &'a Arc<Global>,
	pending_tasks: &'a mut Vec<(PendingTask, tokio::sync::mpsc::Receiver<EventCallback>)>,
	badges: &'a mut HashMap<BadgeId, Badge>,
	paints: &'a mut HashMap<PaintId, Paint>,
	cdn_rename: &'a mut Vec<CdnFileRename>,
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
		cdn_rename,
	} = input;

	let ip = &global.image_processor;

	match cosmetic.data {
		types::CosmeticData::Badge { tooltip, tag } => {
			let badge_id = cosmetic.id.into();

			let image_data = match tokio::fs::read(format!("local/cosmetics/{}", cosmetic.id)).await {
				Ok(data) => bytes::Bytes::from(data),
				Err(e) => {
					if let io::ErrorKind::NotFound = e.kind() {
						let download_url = format!("https://cdn.7tv.app/badge/{}/2x", cosmetic.id);
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
					pending_tasks.push((PendingTask::Badge(badge_id.into()), rx));
					global.all_tasks.lock().await.insert(id.clone(), tx);
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

			let image_set = ImageSet { input, outputs: vec![] };

			let tags = tag.map(|t| vec![t]).unwrap_or_default();
			badges.insert(
				badge_id.into(),
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

			for file in &["1x", "2x", "3x"] {
				cdn_rename.push(CdnFileRename {
					old_path: format!("badge/{}/{}", cosmetic.id, file),
					new_path: format!("badge/{}/{}.webp", badge_id, file),
				});
			}
		}
		types::CosmeticData::Paint { data, drop_shadows } => {
			let paint_id = cosmetic.id.into();

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
							pending_tasks.push((PendingTask::Paint(paint_id.into(), layer_id), rx));
							global.all_tasks.lock().await.insert(id.clone(), tx);
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
				paint_id.into(),
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
