use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use shared::database::queries::{filter, update};
use shared::database::role::permissions::{
	AdminPermission, EmotePermission, EmoteSetPermission, FlagPermission, Permissions, PermissionsExt, UserPermission,
};
use shared::database::stored_event::StoredEventUserBanData;
use shared::database::user::ban::UserBan;
use shared::database::user::User;
use shared::event::{InternalEvent, InternalEventData};
use shared::old_types::object_id::GqlObjectId;
use shared::old_types::BanEffect;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::middleware::session::Session;
use crate::http::v3::gql::guards::PermissionGuard;
use crate::http::v3::gql::queries::user::{User as GqlUser, UserPartial};
use crate::transactions::{with_transaction, TransactionError};

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
		#[graphql(validator(min_length = 1, max_length = 500))] reason: String,
		effects: BanEffect,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
		_anonymous: Option<bool>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let authed_user = session.user()?;

		// check if victim exists
		let victim = global
			.user_loader
			.load(global, victim_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		if authed_user.id == victim.id {
			return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "cannot ban yourself"));
		} else if authed_user.computed.highest_role_rank <= victim.computed.highest_role_rank
			&& !authed_user.has(AdminPermission::SuperAdmin)
		{
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"can only ban users with lower roles",
			));
		}

		let res = with_transaction(global, |mut tx| async move {
			let ban = UserBan {
				id: Default::default(),
				user_id: victim.id,
				template_id: None,
				expires_at: expire_at,
				created_by_id: authed_user.id,
				reason,
				tags: vec![],
				removed: None,
				permissions: ban_effect_to_permissions(effects),
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			};

			let res = tx
				.update_one(
					filter::filter! {
						User {
							#[query(rename = "_id")]
							id: victim.id,
						}
					},
					update::update! {
						#[query(set)]
						User {
							has_bans: true,
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					None,
				)
				.await?;

			tx.insert_one::<UserBan>(&ban, None).await?;

			if res.modified_count > 0 {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::UserBan {
						after: ban.clone(),
						data: StoredEventUserBanData::Ban,
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			Ok(ban)
		})
		.await;

		match res {
			Ok(ban) => Ok(Some(Ban::from_db(victim_id, ban))),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(guard = "PermissionGuard::one(UserPermission::Moderate)")]
	async fn edit_ban<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		ban_id: GqlObjectId,
		#[graphql(validator(min_length = 1, max_length = 500))] reason: Option<String>,
		effects: Option<BanEffect>,
		expire_at: Option<chrono::DateTime<chrono::Utc>>,
	) -> Result<Option<Ban>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;
		let authed_user = session.user()?;

		let res = with_transaction(global, |mut tx| async move {
			let ban_before = tx
				.find_one_and_update(
					filter::filter! {
						UserBan {
							#[query(rename = "_id")]
							id: ban_id.id(),
						}
					},
					update::update! {
						#[query(set)]
						UserBan {
							#[query(optional)]
							reason: reason.clone(),
							#[query(optional)]
							expires_at: expire_at,
							#[query(optional, serde)]
							permissions: effects.map(ban_effect_to_permissions),
							updated_at: chrono::Utc::now(),
							search_updated_at: &None,
						}
					},
					FindOneAndUpdateOptions::builder()
						.return_document(ReturnDocument::Before)
						.build(),
				)
				.await?
				.ok_or_else(|| TransactionError::Custom(ApiError::not_found(ApiErrorCode::LoadError, "ban not found")))?;

			let mut ban_after = ban_before.clone();

			if let Some(reason) = reason.clone() {
				ban_after.reason = reason;
			}

			if let Some(effects) = effects {
				ban_after.permissions = ban_effect_to_permissions(effects);
			}

			if let Some(expire_at) = expire_at {
				ban_after.expires_at = Some(expire_at);
			}

			if reason.is_some() {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::UserBan {
						after: ban_after.clone(),
						data: StoredEventUserBanData::ChangeReason {
							old: ban_before.reason,
							new: ban_after.reason.clone(),
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if effects.is_some() {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::UserBan {
						after: ban_after.clone(),
						data: StoredEventUserBanData::ChangeUserBanPermissions {
							old: Box::new(ban_before.permissions),
							new: Box::new(ban_after.permissions.clone()),
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			if expire_at.is_some() {
				tx.register_event(InternalEvent {
					actor: Some(authed_user.clone()),
					session_id: session.user_session_id(),
					data: InternalEventData::UserBan {
						after: ban_after.clone(),
						data: StoredEventUserBanData::ChangeExpiresAt {
							old: ban_before.expires_at,
							new: ban_after.expires_at,
						},
					},
					timestamp: chrono::Utc::now(),
				})?;
			}

			Ok(ban_after)
		})
		.await;

		match res {
			Ok(ban) => Ok(Some(Ban::from_db(ban.user_id.into(), ban))),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
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
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load_fast(global, self.victim_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.map(|u| UserPartial::from_db(global, u))
			.unwrap_or_else(UserPartial::deleted_user)
			.into())
	}

	async fn actor<'ctx>(&self, ctx: &Context<'ctx>) -> Result<GqlUser, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(global
			.user_loader
			.load(global, self.actor_id.id())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
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
			victim_id: user_id,
			actor_id: ban.created_by_id.into(),
		}
	}
}
