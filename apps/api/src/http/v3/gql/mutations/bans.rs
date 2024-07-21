use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use hyper::StatusCode;
use mongodb::options::ReturnDocument;
use shared::database::audit_log::{AuditLog, AuditLogData, AuditLogUserData};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{
	AdminPermission, EmotePermission, EmoteSetPermission, FlagPermission, Permissions, PermissionsExt, UserPermission,
};
use shared::database::user::ban::UserBan;
use shared::database::user::User;
use shared::database::MongoCollection;
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::BanEffect;

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::middleware::auth::AuthSession;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::user::{User as GqlUser, UserPartial};

#[derive(Default)]
pub struct BansMutation;

fn ban_effect_to_permissions(effects: BanEffect) -> Permissions {
	let mut perms = Permissions::default();

	if effects.contains(BanEffect::MemoryHole) {
		// Hide the user because of memory hole
		perms.allow(FlagPermission::Hidden);
	}

	if effects.contains(BanEffect::NoAuth) | effects.contains(BanEffect::BlockedIp) {
		// Remove all permissions
		perms.deny(UserPermission::Login);
	}

	if effects.contains(BanEffect::NoPermissions) {
		// Remove all permissions
		perms.deny(EmotePermission::all_flags());
		perms.deny(EmoteSetPermission::all_flags());
		perms.deny(UserPermission::all_flags() & !UserPermission::Login);
	}

	perms
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl BansMutation {
	#[graphql(guard = "PermissionGuard::one(UserPermission::Moderate)")]
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
			.user_loader
			.load(global, victim_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "user not found"))?;

		let actor = auth_session.user(&global).await?;
		if actor.id == victim.id {
			return Err(ApiError::new_const(StatusCode::BAD_REQUEST, "cannot ban yourself"));
		} else if actor.computed.highest_role_rank <= victim.computed.highest_role_rank
			&& !actor.has(AdminPermission::SuperAdmin)
		{
			return Err(ApiError::new_const(
				StatusCode::FORBIDDEN,
				"can only ban users with lower roles",
			));
		}

		let ban = UserBan {
			id: Default::default(),
			user_id: victim.id,
			template_id: None,
			expires_at: expire_at,
			created_by_id: actor.id,
			reason,
			tags: vec![],
			removed: None,
			permissions: ban_effect_to_permissions(effects),
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		};

		let mut session = global.mongo.start_session().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start session");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		session.start_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to start transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		let res = User::collection(&global.db)
			.update_one(
				filter::filter! {
					User {
						#[filter(rename = "_id")]
						id: victim.id,
					}
				},
				update::update! {
					#[update(set)]
					User {
						has_bans: true,
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update user permissions");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		UserBan::collection(&global.db)
			.insert_one(&ban)
			.session(&mut session)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to insert user ban");
				ApiError::INTERNAL_SERVER_ERROR
			})?;

		if res.modified_count > 0 {
			AuditLog::collection(&global.db)
				.insert_one(AuditLog {
					id: Default::default(),
					actor_id: Some(actor.id),
					data: AuditLogData::User {
						target_id: victim.id,
						data: AuditLogUserData::Ban,
					},
					updated_at: chrono::Utc::now(),
					search_updated_at: None,
				})
				.session(&mut session)
				.await
				.map_err(|e| {
					tracing::error!(error = %e, "failed to insert audit log");
					ApiError::INTERNAL_SERVER_ERROR
				})?;
		}

		session.commit_transaction().await.map_err(|err| {
			tracing::error!(error = %err, "failed to commit transaction");
			ApiError::INTERNAL_SERVER_ERROR
		})?;

		Ok(Some(Ban::from_db(victim_id, ban)))
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Moderate)")]
	async fn edit_ban<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		ban_id: GqlObjectId,
		reason: Option<String>,
		effects: Option<BanEffect>,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let ban = UserBan::collection(&global.db)
			.find_one_and_update(
				filter::filter! {
					UserBan {
						#[filter(rename = "_id")]
						id: ban_id.id(),
					}
				},
				update::update! {
					#[update(set)]
					UserBan {
						#[update(optional)]
						reason,
						/// TODO(lennart): how do you convert a ban from a temporary to a permanent ban?
						/// This value is optional but a null value means that the ban is permanent, and also means no change.
						#[update(optional)]
						expires_at: expire_at,
						#[update(optional)]
						permissions: effects.map(ban_effect_to_permissions),
						updated_at: chrono::Utc::now(),
					}
				},
			)
			.return_document(ReturnDocument::After)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to update user ban");
				ApiError::INTERNAL_SERVER_ERROR
			})?
			.ok_or(ApiError::new_const(StatusCode::NOT_FOUND, "ban not found"))?;

		Ok(Some(Ban::from_db(ban.user_id.into(), ban)))
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

		Ok(global
			.user_loader
			.load_fast(global, self.victim_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user)
			.into())
	}

	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<GqlUser, ApiError> {
		let global: &Arc<Global> = ctx.data().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(global
			.user_loader
			.load(global, self.actor_id.id())
			.await
			.map_err(|()| ApiError::INTERNAL_SERVER_ERROR)?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user)
			.into())
	}
}

impl Ban {
	fn from_db(user_id: GqlObjectId, ban: UserBan) -> Self {
		let mut effects = BanEffect::none();

		if ban.permissions.has(FlagPermission::Hidden) {
			effects |= BanEffect::MemoryHole;
		}

		if ban.permissions.has(UserPermission::Login) {
			effects |= BanEffect::NoAuth;
		}

		if ban.permissions.has_all([
			EmotePermission::all_flags().into(),
			EmoteSetPermission::all_flags().into(),
			(UserPermission::all_flags() & !UserPermission::Login).into(),
		]) {
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
