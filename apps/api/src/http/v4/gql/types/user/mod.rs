use std::sync::Arc;

use async_graphql::{ComplexObject, Context, SimpleObject};
use shared::database::emote_set::{EmoteSetId, EmoteSetKind};
use shared::database::product::SubscriptionProductId;
use shared::database::role::permissions::{PermissionsExt, UserPermission};
use shared::database::role::RoleId;
use shared::database::user::editor::EditorEmoteSetPermission;
use shared::database::user::UserId;
use shared::typesense::types::event::EventId;

use super::raw_entitlement::RawEntitlements;
use super::{Color, Emote, EmoteSet, Event, Permissions, Role, UserEditor, UserEvent};
use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::RateLimitGuard;
use crate::http::middleware::session::Session;
use crate::search::{search, sorted_results, SearchOptions};

pub mod billing;
pub mod connection;
pub mod inventory;
pub mod style;

pub use connection::*;
pub use inventory::*;
pub use style::*;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
	pub id: UserId,
	pub connections: Vec<UserConnection>,
	pub updated_at: chrono::DateTime<chrono::Utc>,
	pub search_updated_at: Option<chrono::DateTime<chrono::Utc>>,

	// Computed fields
	pub highest_role_rank: i32,
	pub highest_role_color: Option<Color>,
	pub role_ids: Vec<RoleId>,

	#[graphql(skip)]
	full_user: shared::database::user::FullUser,
}

#[ComplexObject]
impl User {
	#[tracing::instrument(skip_all, name = "User::main_connection")]
	async fn main_connection(&self) -> Option<&UserConnection> {
		self.connections.first()
	}

	// TODO: Does it make sense to paginate this?
	#[tracing::instrument(skip_all, name = "User::owned_emotes")]
	async fn owned_emotes(&self, ctx: &Context<'_>) -> Result<Vec<Emote>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut emotes = global
			.emote_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emotes"))?
			.unwrap_or_default();

		emotes.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emotes
			.into_iter()
			.map(|e| Emote::from_db(e, &global.config.api.cdn_origin))
			.collect())
	}

	#[tracing::instrument(skip_all, name = "User::owned_emote_sets")]
	async fn owned_emote_sets(&self, ctx: &Context<'_>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default();

		emote_sets.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emote_sets.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "User::personal_emote_set")]
	async fn personal_emote_set(&self, ctx: &Context<'_>) -> Result<Option<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default();

		Ok(emote_sets
			.into_iter()
			.find(|e| e.kind == EmoteSetKind::Personal)
			.map(Into::into))
	}

	#[tracing::instrument(skip_all, name = "User::special_emote_sets")]
	async fn special_emote_sets(&self, ctx: &Context<'_>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_sets = global
			.emote_set_by_id_loader
			.load_many(self.full_user.computed.entitlements.emote_sets.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?;

		let mut emote_sets = emote_sets.into_values().collect::<Vec<_>>();
		emote_sets.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emote_sets.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "User::special_emote_sets")]
	async fn emote_sets(&self, ctx: &Context<'_>) -> Result<Vec<EmoteSet>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let owned_emote_sets = global
			.emote_set_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.unwrap_or_default();

		let mut emote_sets: Vec<_> = global
			.emote_set_by_id_loader
			.load_many(self.full_user.computed.entitlements.emote_sets.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.into_values()
			.chain(owned_emote_sets.into_iter())
			.collect();

		emote_sets.sort_by(|a, b| a.id.cmp(&b.id));

		Ok(emote_sets.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "User::style")]
	async fn style(&self, ctx: &Context<'_>) -> Result<UserStyle, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		Ok(UserStyle::from_user(global, &self.full_user))
	}

	#[tracing::instrument(skip_all, name = "User::roles")]
	async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let mut loaded = global
			.role_by_id_loader
			.load_many(self.role_ids.iter().copied())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load roles"))?;

		let mut roles = Vec::with_capacity(loaded.len());

		for id in &self.role_ids {
			if let Some(role) = loaded.remove(id) {
				roles.push(role);
			}
		}

		Ok(roles.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "User::permissions")]
	async fn permissions(&self, ctx: &Context<'_>) -> Result<Permissions, ApiError> {
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.id && !authed_user.has(UserPermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you are not allowed to see this user's permissions",
			));
		}

		Ok(Permissions::from(self.full_user.computed.permissions.clone()))
	}

	#[tracing::instrument(skip_all, name = "User::editors")]
	async fn editors(&self, ctx: &Context<'_>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let editors = global
			.user_editor_by_user_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
			.unwrap_or_default();

		Ok(editors.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "User::editor_for")]
	async fn editor_for(&self, ctx: &Context<'_>) -> Result<Vec<UserEditor>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let editors = global
			.user_editor_by_editor_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
			.unwrap_or_default();

		Ok(editors.into_iter().map(Into::into).collect())
	}

	#[tracing::instrument(skip_all, name = "User::editable_emote_set_ids")]
	async fn editable_emote_set_ids(&self, ctx: &Context<'_>) -> Result<Vec<EmoteSetId>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.id && !authed_user.has(UserPermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you are not allowed to see this user's emote sets",
			));
		}

		let owners = global
			.user_editor_by_editor_id_loader
			.load(self.id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editors"))?
			.unwrap_or_default()
			.into_iter()
			.filter(|editor| editor.permissions.has_emote_set(EditorEmoteSetPermission::Manage))
			.map(|editor| editor.id.user_id)
			.chain(std::iter::once(self.id));

		let mut emote_sets: Vec<EmoteSetId> = global
			.emote_set_by_user_id_loader
			.load_many(owners)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote sets"))?
			.into_values()
			.flatten()
			.map(|e| e.id)
			.collect();

		emote_sets.sort();

		Ok(emote_sets)
	}

	#[graphql(guard = "RateLimitGuard::search(1)")]
	#[tracing::instrument(skip_all, name = "User::events")]
	async fn events<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(maximum = 10))] page: Option<u32>,
		#[graphql(validator(minimum = 1, maximum = 100))] per_page: Option<u32>,
	) -> Result<Vec<UserEvent>, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let authed_user = session.user()?;

		if authed_user.id != self.id && !authed_user.has(UserPermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you are not allowed to see this user's events",
			));
		}

		let options = SearchOptions::builder()
			.query("*".to_owned())
			.filter_by(format!("target_id: {}", EventId::User(self.id)))
			.sort_by(vec!["created_at:desc".to_owned()])
			.page(page)
			.per_page(per_page.unwrap_or(20))
			.build();

		let result = search::<shared::typesense::types::event::Event>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let events = global
			.event_by_id_loader
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| {
				tracing::error!("failed to load event");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load event")
			})?;

		Ok(sorted_results(result.hits, events)
			.into_iter()
			.filter_map(|e| Event::try_from(e).ok())
			.collect())
	}

	#[tracing::instrument(skip_all, name = "User::inventory")]
	async fn inventory(&self, ctx: &Context<'_>) -> Result<UserInventory, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let full_user = global
			.user_loader
			.load_user(global, self.full_user.user.clone())
			.await
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?;

		Ok(UserInventory::from_user(&full_user))
	}

	#[tracing::instrument(skip_all, name = "User::billing")]
	async fn billing(&self, product_id: SubscriptionProductId) -> Result<billing::Billing, ApiError> {
		Ok(billing::Billing {
			user_id: self.id,
			product_id,
		})
	}

	async fn raw_entitlements(&self, ctx: &Context<'_>) -> Result<RawEntitlements, ApiError> {
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let authed_user = session.user()?;

		if !authed_user.has(UserPermission::ManageAny) {
			return Err(ApiError::forbidden(
				ApiErrorCode::LackingPrivileges,
				"you are not allowed to see this user's entitlements",
			));
		}

		Ok(RawEntitlements::from_db(
			self.full_user
				.computed
				.raw_entitlements
				.as_ref()
				.unwrap_or(&Default::default()),
		))
	}
}

impl From<shared::database::user::FullUser> for User {
	fn from(value: shared::database::user::FullUser) -> Self {
		Self {
			id: value.id,
			connections: value.connections.iter().cloned().map(Into::into).collect(),
			updated_at: value.updated_at,
			search_updated_at: value.search_updated_at,
			highest_role_rank: value.computed.highest_role_rank,
			highest_role_color: value.computed.highest_role_color.map(Color),
			role_ids: value.computed.roles.clone(),
			full_user: value,
		}
	}
}
