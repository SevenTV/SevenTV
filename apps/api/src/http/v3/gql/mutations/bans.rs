use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use hyper::StatusCode;
use mongodb::bson::{doc, to_bson};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::role::permissions::{
	EmotePermission, EmoteSetPermission, FeaturePermission, Permissions, UserPermission,
};
use shared::database::user::ban::UserBan;
use shared::database::user::User;
use shared::database::{Collection, Id};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::BanEffect;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::{User as GqlUser, UserPartial};

#[derive(Default)]
pub struct BansMutation;

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl BansMutation {
	#[graphql(guard = "PermissionGuard::one(UserPermission::Ban)")]
	async fn create_ban<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		victim_id: GqlObjectId,
		reason: String,
		effects: BanEffect,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
		_anonymous: Option<bool>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		let auth_session = ctx.data::<AuthSession>().map_err(|_| ApiError::UNAUTHORIZED)?;

		// check if victim exists
		let victim = global
			.user_loader()
			.load(global, victim_id.0.cast())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "user not found"))?;

		let actor = auth_session.user(&global).await?;
		if actor.id == victim.id {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "cannot ban yourself"));
		} else if actor.computed.highest_role_rank <= victim.computed.highest_role_rank {
			return Err(ApiError::new_const(
				StatusCode::FORBIDDEN,
				"can only ban users with lower roles",
			));
		}

		let ban = UserBan {
			id: Default::default(),
			template_id: None,
			expires_at: expire_at,
			created_by_id: actor.id,
			reason,
			tags: vec![],
			removed: None,
			permissions: {
				let mut perms = Permissions::default();

				if effects.contains(BanEffect::MemoryHole) {
					// Hide the user because of memory hole
					perms.allow(UserPermission::Hidden);
				}

				if effects.contains(BanEffect::NoAuth) | effects.contains(BanEffect::BlockedIp) {
					// Remove all permissions
					perms.deny(UserPermission::Login);
				}

				if effects.contains(BanEffect::NoPermissions) {
					// Remove all permissions
					perms.deny(EmotePermission::all_flags());
					perms.deny(EmoteSetPermission::all_flags());
					perms.deny(FeaturePermission::all_flags());
				}

				perms
			},
		};

		User::collection(global.db())
			.update_one(
				doc! { "_id": victim.id },
				doc! {
					"$push": {
						"bans": to_bson(&ban).unwrap(),
					},
					"$set": {
						"search_index.self_dirty": Id::<()>::new(),
					},
				},
				None,
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update user permissions");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		Ok(Some(Ban::from_db(victim_id, ban)))
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Ban)")]
	async fn edit_ban<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		ban_id: GqlObjectId,
		reason: Option<String>,
		effects: Option<BanEffect>,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let mut update = doc! {};

		if let Some(reason) = reason {
			update.insert("bans.$.reason", reason);
		}

		if let Some(expire_at) = expire_at {
			update.insert("bans.$.expires_at", expire_at);
		}

		if let Some(effects) = effects {
			let mut perms = Permissions::default();

			if effects.contains(BanEffect::MemoryHole) {
				// Hide the user because of memory hole
				perms.allow(UserPermission::Hidden);
			}

			if effects.contains(BanEffect::NoAuth) | effects.contains(BanEffect::BlockedIp) {
				// Remove all permissions
				perms.deny(UserPermission::Login);
			}

			if effects.contains(BanEffect::NoPermissions) {
				// Remove all permissions
				perms.deny(EmotePermission::all_flags());
				perms.deny(EmoteSetPermission::all_flags());
				perms.deny(FeaturePermission::all_flags());
			}

			update.insert("bans.$.permissions", to_bson(&perms).unwrap());
		}

		update.insert("search_index.self_dirty", Id::<()>::new());

		let user = User::collection(global.db())
			.find_one_and_update(
				doc! { "bans.id": ban_id.0 },
				doc! { "$set": update },
				FindOneAndUpdateOptions::builder()
					.return_document(ReturnDocument::After)
					.projection(doc! { "search_index": 0 })
					.build(),
			)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update user ban");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "ban not found"))?;

		Ok(Some(Ban::from_db(
			user.id.into(),
			user.bans.iter().find(|ban| ban.id == ban_id.0.cast()).unwrap().clone(),
		)))
	}
}

#[derive(SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Ban {
	id: GqlObjectId,
	reason: String,
	effects: BanEffect,
	expire_at: chrono::DateTime<chrono::Utc>,
	created_at: chrono::DateTime<chrono::Utc>,
	victim_id: GqlObjectId,
	// victim
	actor_id: GqlObjectId,
	// actor
}

#[ComplexObject(rename_fields = "camelCase", rename_args = "snake_case")]
impl Ban {
	async fn victim<'ctx>(&self, ctx: &Context<'ctx>) -> Result<GqlUser, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, self.victim_id.0.cast()).await?.unwrap_or_else(UserPartial::deleted_user).into())
	}

	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<GqlUser, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;
		Ok(UserPartial::load_from_db(global, self.actor_id.0.cast()).await?.unwrap_or_else(UserPartial::deleted_user).into())
	}
}

impl Ban {
	fn from_db(user_id: GqlObjectId, ban: UserBan) -> Self {
		let mut effects = BanEffect::none();

		if ban.permissions.has(UserPermission::Hidden) {
			effects |= BanEffect::MemoryHole;
		}

		if ban.permissions.denied(UserPermission::Login) {
			effects |= BanEffect::NoAuth;
		}

		if ban.permissions.denied(EmotePermission::all_flags())
			&& ban.permissions.denied(EmoteSetPermission::all_flags())
			&& ban.permissions.denied(FeaturePermission::all_flags())
		{
			effects |= BanEffect::NoPermissions;
		}

		Self {
			id: ban.id.into(),
			reason: ban.reason,
			effects,
			expire_at: ban.expires_at.unwrap_or_default(),
			created_at: ban.id.timestamp(),
			victim_id: user_id.into(),
			actor_id: ban.created_by_id.into(),
		}
	}
}
