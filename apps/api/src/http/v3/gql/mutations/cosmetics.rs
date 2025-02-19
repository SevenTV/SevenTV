use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use mongodb::options::{FindOneAndUpdateOptions, FindOneOptions, ReturnDocument};
use shared::database::badge::BadgeId;
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::paint::{
	Paint, PaintData, PaintGradientStop, PaintId, PaintLayer, PaintLayerId, PaintLayerType, PaintShadow,
};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::PaintPermission;
use shared::database::stored_event::StoredEventPaintData;
use shared::event::{InternalEvent, InternalEventData};
use shared::old_types::cosmetic::{CosmeticPaintFunction, CosmeticPaintModel, CosmeticPaintShape};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::middleware::session::Session;
use crate::http::validators::NameValidator;
use crate::transactions::{transaction, transaction_with_mutex, GeneralMutexKey, TransactionError};

#[derive(Default)]
pub struct CosmeticsMutation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, async_graphql::Enum)]
pub enum CosmeticKind {
	Paint,
	Badge,
}

#[derive(SimpleObject)]
pub struct CosmeticReprocessResults {
	pub paints: Vec<CosmeticReprocessResult>,
	pub badges: Vec<CosmeticReprocessResult>,
}

#[derive(SimpleObject)]
pub struct CosmeticReprocessResult {
	pub id: GqlObjectId,
	pub success: bool,
	pub error: Option<String>,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl CosmeticsMutation {
	#[graphql(guard = "PermissionGuard::one(PaintPermission::Manage)")]
	#[tracing::instrument(skip_all, name = "CosmeticsMutation::create_cosmetic_paint")]
	async fn create_cosmetic_paint<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		definition: CosmeticPaintInput,
	) -> Result<GqlObjectId, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let id = PaintId::new();

		let paint = Paint {
			id,
			name: definition.name.clone(),
			data: definition.into_db(id, global).await?,
			search_updated_at: None,
			updated_at: chrono::Utc::now(),
			..Default::default()
		};

		let res = transaction::<(), (), _, _>(global, |mut tx| async move {
			tx.insert_one(paint.clone(), None).await?;

			tx.register_event(InternalEvent {
				actor: Some(authed_user.clone()),
				session_id: session.user_session_id(),
				data: InternalEventData::Paint {
					after: paint,
					data: StoredEventPaintData::Create,
				},
				timestamp: chrono::Utc::now(),
			})?;

			Ok(())
		})
		.await;

		match res {
			Ok(_) => Ok(id.into()),
			Err(e) => {
				tracing::error!(error = %e, "failed to insert paint");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"failed to insert paint",
				))
			}
		}
	}

	async fn cosmetics(&self, id: GqlObjectId) -> CosmeticOps {
		CosmeticOps { id }
	}

	#[graphql(guard = "PermissionGuard::one(PaintPermission::Manage)")]
	async fn reprocess_cosmetic_image(
		&self,
		ctx: &Context<'_>,
		paint_ids: Option<Vec<GqlObjectId>>,
		badge_ids: Option<Vec<GqlObjectId>>,
	) -> Result<CosmeticReprocessResults, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut paint_ids = paint_ids
			.unwrap_or_default()
			.into_iter()
			.map(|id| id.id())
			.collect::<Vec<PaintId>>();
		let mut badge_ids = badge_ids
			.unwrap_or_default()
			.into_iter()
			.map(|id| id.id())
			.collect::<Vec<BadgeId>>();

		paint_ids.sort();
		paint_ids.dedup();
		badge_ids.sort();
		badge_ids.dedup();

		let paints = global
			.paint_by_id_loader
			.load_many(paint_ids.iter().copied())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load paint"))?;

		let badges = global
			.badge_by_id_loader
			.load_many(badge_ids.iter().copied())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load badge"))?;

		let mut paint_responses = Vec::new();

		for id in paint_ids {
			let Some(paint) = paints.get(&id) else {
				paint_responses.push(CosmeticReprocessResult {
					id: id.into(),
					success: false,
					error: Some("paint not found".to_string()),
				});
				continue;
			};

			let mut at_least_one = false;

			for layer in &paint.data.layers {
				if let PaintLayerType::Image(ImageSet { input, .. }) = &layer.ty {
					let image = match input {
						ImageSetInput::Pending { path, .. } => path.clone(),
						ImageSetInput::Image(image) => image.path.clone(),
					};

					global
						.image_processor
						.reprocess_paint_layer(image, paint.id, layer.id)
						.await
						.map_err(|e| {
							tracing::error!(error = ?e, "failed to reprocess paint layer");
							ApiError::internal_server_error(
								ApiErrorCode::ImageProcessorError,
								"failed to reprocess paint layer",
							)
						})?;

					at_least_one = true;
				}
			}

			paint_responses.push(CosmeticReprocessResult {
				id: id.into(),
				success: at_least_one,
				error: if at_least_one {
					None
				} else {
					Some("no image layers to reprocess".to_string())
				},
			});
		}

		let mut badge_responses = Vec::new();

		for id in badge_ids {
			let Some(badge) = badges.get(&id) else {
				badge_responses.push(CosmeticReprocessResult {
					id: id.into(),
					success: false,
					error: Some("badge not found".to_string()),
				});
				continue;
			};

			let image = match &badge.image_set.input {
				ImageSetInput::Pending { path, .. } => path.clone(),
				ImageSetInput::Image(image) => image.path.clone(),
			};

			global.image_processor.reprocess_badge(image, badge.id).await.map_err(|e| {
				tracing::error!(error = ?e, "failed to reprocess badge");
				ApiError::internal_server_error(ApiErrorCode::ImageProcessorError, "failed to reprocess badge")
			})?;

			badge_responses.push(CosmeticReprocessResult {
				id: id.into(),
				success: true,
				error: None,
			});
		}

		Ok(CosmeticReprocessResults {
			paints: paint_responses,
			badges: badge_responses,
		})
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintInput {
	#[graphql(validator(custom = "NameValidator"))]
	name: String,
	function: CosmeticPaintFunction,
	color: Option<i32>,
	#[graphql(validator(minimum = 0, maximum = 360))]
	angle: Option<u32>,
	shape: Option<CosmeticPaintShape>,
	image_url: Option<String>,
	repeat: bool,
	stops: Vec<CosmeticPaintStopInput>,
	shadows: Vec<CosmeticPaintShadowInput>,
}

impl CosmeticPaintInput {
	async fn into_db(self, paint_id: PaintId, global: &Arc<Global>) -> Result<PaintData, ApiError> {
		let layer_id = PaintLayerId::new();

		let ty = match self.function {
			CosmeticPaintFunction::LinearGradient => {
				let stops = self
					.stops
					.iter()
					.map(|stop| PaintGradientStop {
						at: stop.at,
						color: stop.color,
					})
					.collect();

				PaintLayerType::LinearGradient {
					angle: self.angle.unwrap_or(0) as i32,
					repeating: self.repeat,
					stops,
				}
			}
			CosmeticPaintFunction::RadialGradient => {
				let stops = self
					.stops
					.iter()
					.map(|stop| PaintGradientStop {
						at: stop.at,
						color: stop.color,
					})
					.collect();

				PaintLayerType::RadialGradient {
					repeating: self.repeat,
					shape: self.shape.unwrap_or(CosmeticPaintShape::Ellipse).into(),
					stops,
				}
			}
			CosmeticPaintFunction::Url => {
				let Some(image_url) = self.image_url.and_then(|u| url::Url::parse(&u).ok()) else {
					return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "invalid image url"));
				};

				// TODO(troy): This allows for anyone to pass any url and we will blindly do a
				// GET request against it We need to make sure the URL does not go to any
				// internal services or other places that we don't want and we need to make
				// sure that the file isnt too big.
				let image_data = match global.http_client.get(image_url).send().await {
					Ok(res) if res.status().is_success() => match res.bytes().await {
						Ok(bytes) => bytes,
						Err(e) => {
							tracing::error!(error = ?e, "failed to read image data");
							return Err(ApiError::internal_server_error(
								ApiErrorCode::BadRequest,
								"failed to read image data",
							));
						}
					},
					Ok(res) => {
						tracing::error!(status = ?res.status(), "failed to request image url");
						return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "failed to request image url"));
					}
					Err(e) => {
						tracing::error!(error = ?e, "failed to request image url");
						return Err(ApiError::internal_server_error(
							ApiErrorCode::BadRequest,
							"failed to request image url",
						));
					}
				};

				let input = match global
					.image_processor
					.upload_paint_layer(paint_id, layer_id, image_data)
					.await
				{
					Ok(image_processor_proto::ProcessImageResponse { error: Some(error), .. }) => {
						tracing::error!(error = ?error, "failed to start processing image");
						return Err(ApiError::internal_server_error(
							ApiErrorCode::ImageProcessorError,
							"image processor error",
						));
					}
					Ok(image_processor_proto::ProcessImageResponse {
						id,
						upload_info:
							Some(image_processor_proto::ProcessImageResponseUploadInfo {
								path: Some(path),
								content_type,
								size,
							}),
						error: None,
					}) => ImageSetInput::Pending {
						task_id: id,
						path: path.path,
						mime: content_type,
						size: size as i64,
					},
					Err(e) => {
						tracing::error!(error = ?e, "failed to start send image processor request");
						return Err(ApiError::internal_server_error(
							ApiErrorCode::ImageProcessorError,
							"image processor error",
						));
					}
					_ => {
						return Err(ApiError::internal_server_error(
							ApiErrorCode::ImageProcessorError,
							"image processor error",
						));
					}
				};

				PaintLayerType::Image(ImageSet { input, outputs: vec![] })
			}
		};

		let layer = PaintLayer {
			id: layer_id,
			ty,
			..Default::default()
		};

		Ok(PaintData {
			layers: vec![layer],
			shadows: self
				.shadows
				.iter()
				.map(|shadow| shadow.try_into())
				.collect::<Result<_, _>>()?,
		})
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintStopInput {
	#[graphql(validator(minimum = 0, maximum = 1))]
	at: f64,
	color: i32,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintShadowInput {
	x_offset: String,
	y_offset: String,
	radius: String,
	color: i32,
}

impl TryFrom<&CosmeticPaintShadowInput> for PaintShadow {
	type Error = ApiError;

	fn try_from(value: &CosmeticPaintShadowInput) -> Result<Self, Self::Error> {
		Ok(PaintShadow {
			color: value.color,
			offset_x: value
				.x_offset
				.parse::<f64>()
				.map_err(|e| ApiError::bad_request(ApiErrorCode::BadRequest, e.to_string()))?,
			offset_y: value
				.y_offset
				.parse::<f64>()
				.map_err(|e| ApiError::bad_request(ApiErrorCode::BadRequest, e.to_string()))?,
			blur: value
				.radius
				.parse::<f64>()
				.map_err(|e| ApiError::bad_request(ApiErrorCode::BadRequest, e.to_string()))?,
		})
	}
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct CosmeticOps {
	id: GqlObjectId,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl CosmeticOps {
	#[graphql(guard = "PermissionGuard::one(PaintPermission::Manage)")]
	#[tracing::instrument(skip_all, name = "CosmeticOps::update_paint")]
	async fn update_paint<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		definition: CosmeticPaintInput,
	) -> Result<CosmeticPaintModel, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let name = definition.name.clone();
		let data = definition.into_db(self.id.id(), global).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::Paint(self.id.id()).into()),
			|mut tx| async move {
				let before_paint = tx
					.find_one(
						filter::filter! {
							Paint {
								#[query(rename = "_id")]
								id: self.id.id(),
							}
						},
						FindOneOptions::default(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "paint not found"))
					})?;

				if before_paint.data.layers.iter().any(|l| match &l.ty {
					PaintLayerType::Image(ImageSet { input, .. }) => input.is_pending(),
					_ => false,
				}) {
					return Err(TransactionError::Custom(ApiError::bad_request(
						ApiErrorCode::BadRequest,
						"cannot update paint with pending image layers",
					)));
				}

				let after_paint = tx
					.find_one_and_update(
						filter::filter! {
							Paint {
								#[query(rename = "_id")]
								id: self.id.id(),
							}
						},
						update::update! {
							#[query(set)]
							Paint {
								name: &name,
								#[query(serde)]
								data: &data,
								updated_at: chrono::Utc::now(),
								search_updated_at: &None,
							}
						},
						FindOneAndUpdateOptions::builder()
							.return_document(ReturnDocument::After)
							.build(),
					)
					.await?
					.ok_or_else(|| {
						TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "paint not found"))
					})?;

				if before_paint.name != after_paint.name {
					tx.register_event(InternalEvent {
						actor: Some(authed_user.clone()),
						session_id: session.user_session_id(),
						data: InternalEventData::Paint {
							after: after_paint.clone(),
							data: StoredEventPaintData::ChangeName {
								old: before_paint.name,
								new: after_paint.name.clone(),
							},
						},
						timestamp: chrono::Utc::now(),
					})?;
				}

				if before_paint.data != after_paint.data {
					tx.register_event(InternalEvent {
						actor: Some(authed_user.clone()),
						session_id: session.user_session_id(),
						data: InternalEventData::Paint {
							after: after_paint.clone(),
							data: StoredEventPaintData::ChangeData {
								old: before_paint.data,
								new: after_paint.data.clone(),
							},
						},
						timestamp: chrono::Utc::now(),
					})?;
				}

				Ok(after_paint)
			},
		)
		.await;

		match res {
			Ok(paint) => Ok(CosmeticPaintModel::from_db(paint, &global.config.api.cdn_origin)),
			Err(e) => {
				tracing::error!(error = %e, "failed to insert paint");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"failed to insert paint",
				))
			}
		}
	}
}
