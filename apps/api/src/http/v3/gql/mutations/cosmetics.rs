use async_graphql::{ComplexObject, InputObject, Object, SimpleObject};
use shared::database::PaintPermission;
use shared::old_types::{CosmeticPaintFunction, CosmeticPaintModel, ObjectId};

use crate::http::error::ApiError;
use crate::http::v3::gql::guards::PermissionGuard;

#[derive(Default)]
pub struct CosmeticsMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl CosmeticsMutation {
	#[graphql(guard = "PermissionGuard::one(PaintPermission::Create)")]
	async fn create_cosmetic_paint(&self, definition: CosmeticPaintInput) -> Result<ObjectId<()>, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}

	async fn cosmetics(&self, id: ObjectId<()>) -> CosmeticOps {
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
	shape: Option<String>,
	image_url: Option<String>,
	repeat: bool,
	stops: Vec<CosmeticPaintStopInput>,
	shadows: Vec<CosmeticPaintShadowInput>,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintStopInput {
	at: f32,
	color: u32,
}

#[derive(InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CosmeticPaintShadowInput {
	x_offset: f32,
	y_offset: f32,
	radius: f32,
	color: u32,
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct CosmeticOps {
	id: ObjectId<()>,
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl CosmeticOps {
	#[graphql(guard = "PermissionGuard::one(PaintPermission::Edit)")]
	async fn update_paint(&self, definition: CosmeticPaintInput) -> Result<CosmeticPaintModel, ApiError> {
		Err(ApiError::NOT_IMPLEMENTED)
	}
}
