use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use mongodb::bson::doc;
use shared::database::{self, Collection, UserId, UserPermission};
use shared::old_types::{BanEffect, BanObjectId, UserObjectId};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::{User, UserPartial};

#[derive(Default)]
pub struct BansMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl BansMutation {
	#[graphql(guard = "PermissionGuard::one(UserPermission::Ban)")]
	async fn create_ban<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		victim_id: UserObjectId,
		reason: String,
		_effects: BanEffect,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
		_anonymous: Option<bool>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		// check if victim exists
		let _ = global
			.user_by_id_loader()
			.load(global, victim_id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::NOT_FOUND)?;

		let ban = database::UserBan {
			user_id: victim_id.id(),
			created_by_id: Some(auth_session.user_id()),
			reason,
			expires_at: expire_at,
			..Default::default()
		};

		database::UserBan::collection(global.db())
			.insert_one(&ban, None)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(Some(Ban::from_db(ban)))
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Ban)")]
	async fn edit_ban<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		ban_id: BanObjectId,
		reason: Option<String>,
		_effects: Option<BanEffect>,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut update = doc! {};

		if let Some(reason) = reason {
			update.insert("reason", reason);
		}

		if let Some(expire_at) = expire_at {
			update.insert("expires_at", expire_at);
		}

		let ban = database::UserBan::collection(global.db())
			.find_one_and_update(doc! { "_id": ban_id.id() }, doc! { "$set": update }, None)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(ban.map(Ban::from_db))
	}
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Ban {
	id: BanObjectId,
	reason: String,
	effects: BanEffect,
	expire_at: chrono::DateTime<chrono::Utc>,
	created_at: chrono::DateTime<chrono::Utc>,
	victim_id: UserObjectId,
	// victim
	actor_id: UserObjectId,
	// actor
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl Ban {
	async fn victim<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, self.victim_id.id()).await?.into())
	}

	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<User, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, self.actor_id.id()).await?.into())
	}
}

impl Ban {
	fn from_db(ban: database::UserBan) -> Self {
		// 11
		let effects = BanEffect::NoPermissions | BanEffect::NoAuth | BanEffect::MemoryHole;

		Self {
			id: ban.id.into(),
			reason: ban.reason,
			effects,
			expire_at: ban.expires_at.unwrap_or_default(),
			created_at: ban.id.timestamp(),
			victim_id: ban.user_id.into(),
			actor_id: ban.created_by_id.unwrap_or(UserId::nil()).into(),
		}
	}
}
