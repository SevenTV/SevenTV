use std::sync::Arc;

use async_graphql::{ComplexObject, Context, InputObject, Object, SimpleObject};
use hyper::StatusCode;
use shared::database::image_set::{ImageSet, ImageSetInput};
use shared::database::paint::{
	Paint, PaintData, PaintGradientStop, PaintId, PaintLayer, PaintLayerId, PaintLayerType, PaintShadow,
};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::PaintPermission;
use shared::database::MongoCollection;
use shared::old_types::cosmetic::{CosmeticPaintFunction, CosmeticPaintModel, CosmeticPaintShape};
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;

#[derive(Default)]
pub struct CosmeticsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl CosmeticsMutation {
	#[graphql(guard = "PermissionGuard::one(PaintPermission::Manage)")]
	async fn create_cosmetic_paint<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		definition: CosmeticPaintInput,
	) -> Result<GqlObjectId, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let id = PaintId::new();

		let paint = Paint {
			id,
			name: definition.name.clone(),
			data: definition.into_db(id, global).await?,
			search_updated_at: None,
			updated_at: chrono::Utc::now(),
			..Default::default()
		};

		Paint::collection(&global.db).insert_one(paint).await.map_err(|e| {
			tracing::error!(error = %e, "failed to insert paint");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(id.into())
	}

	async fn cosmetics(&self, id: GqlObjectId) -> CosmeticOps {
		CosmeticOps { id }
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintInput {
	name: String,
	function: CosmeticPaintFunction,
	color: Option<u32>,
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
					angle: self.angle.unwrap_or(0) as i32,
					repeating: self.repeat,
					shape: self.shape.unwrap_or(CosmeticPaintShape::Ellipse).into(),
					stops,
				}
			}
			CosmeticPaintFunction::Url => {
				let Some(image_url) = self.image_url else {
					return Err(ApiError::BAD_REQUEST);
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
							return Err(ApiError::INTERNAL_SERVER_ERROR);
						}
					},
					Ok(res) => {
						tracing::error!(status = ?res.status(), "failed to request image url");
						return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "failed to request image url"));
					}
					Err(e) => {
						tracing::error!(error = ?e, "failed to request image url");
						return Err(ApiError::INTERNAL_SERVER_ERROR);
					}
				};

				let input = match global
					.image_processor
					.upload_paint_layer(paint_id, layer_id, image_data)
					.await
				{
					Ok(scuffle_image_processor_proto::ProcessImageResponse { error: Some(error), .. }) => {
						tracing::error!(error = ?error, "failed to start processing image");
						return Err(ApiError::new_const(
							StatusCode::INTERNAL_SERVER_ERROR,
							"image processor error",
						));
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
					}) => ImageSetInput::Pending {
						task_id: id,
						path: path.path,
						mime: content_type,
						size,
					},
					Err(e) => {
						tracing::error!(error = ?e, "failed to start send image processor request");
						return Err(ApiError::new_const(
							StatusCode::INTERNAL_SERVER_ERROR,
							"image processor error",
						));
					}
					_ => {
						return Err(ApiError::new_const(
							StatusCode::INTERNAL_SERVER_ERROR,
							"image processor error",
						));
					}
				};

				PaintLayerType::Image(ImageSet {
					input,
					..Default::default()
				})
			}
		};

		let layer = PaintLayer {
			id: layer_id,
			ty,
			..Default::default()
		};

		Ok(PaintData {
			layers: vec![layer],
			shadows: self.shadows.iter().map(|shadow| shadow.to_db()).collect(),
		})
	}
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintStopInput {
	at: f64,
	color: u32,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintShadowInput {
	x_offset: f64,
	y_offset: f64,
	radius: f64,
	color: u32,
}

impl CosmeticPaintShadowInput {
	pub fn to_db(&self) -> PaintShadow {
		PaintShadow {
			color: self.color,
			offset_x: self.x_offset,
			offset_y: self.y_offset,
			blur: self.radius,
		}
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
	async fn update_paint<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		definition: CosmeticPaintInput,
	) -> Result<CosmeticPaintModel, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let _ = global
			.paint_by_id_loader
			.load(self.id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		let name = definition.name.clone();
		let data = definition.into_db(self.id.id(), global).await?;

		let paint = Paint::collection(&global.db)
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
						name,
						#[query(serde)]
						data,
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update paint");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::NOT_FOUND)?;

		CosmeticPaintModel::from_db(paint, &global.config.api.cdn_origin).ok_or(ApiError::INTERNAL_SERVER_ERROR)
	}
}
