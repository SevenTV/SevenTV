use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Object, SimpleObject};
use bitmask_enum::bitmask;
use futures::StreamExt;
use mongodb::bson::doc;
use shared::database::{self, Collection, GlobalConfig, Permissions};
use shared::old_types::RoleObjectId;

use super::users::User;
use crate::global::Global;
use crate::http::error::ApiError;

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/roles.gql

#[derive(Default)]
pub struct RolesQuery;

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct Role {
	id: RoleObjectId,
	name: String,
	color: i32,
	// allowed
	// denied
	position: u32,
	// created_at
	invisible: bool,
	// members
	#[graphql(skip)]
	allowed_db: RolePermission,
	#[graphql(skip)]
	denied_db: RolePermission,
}

#[bitmask(i64)]
// https://github.com/SevenTV/Common/blob/master/structures/v3/type.role.go#L37
pub enum RolePermission {
	CreateEmote = 1 << 0,
	EditEmote = 1 << 1,
	CreateEmoteSet = 1 << 2,
	EditEmoteSet = 1 << 3,

	CreateReport = 1 << 13,
	SendMessages = 1 << 14,

	FeatureZeroWidthEmoteType = 1 << 23,
	FeatureProfilePictureAnimation = 1 << 24,
	FeatureMessagingPriority = 1 << 25,

	ManageBans = 1 << 30,
	ManageRoles = 1 << 31,
	ManageReports = 1 << 32,
	ManageUsers = 1 << 33,

	EditAnyEmote = 1 << 41,
	EditAnyEmoteSet = 1 << 42,

	BypassPrivacy = 1 << 48,

	ManageContent = 1 << 54,
	ManageStack = 1 << 55,
	ManageCosmetics = 1 << 56,
	RunJobs = 1 << 57,
	ManageEntitlements = 1 << 58,

	SuperAdministrator = 1 << 62,
}

impl RolePermission {
	fn from_db(value: Permissions) -> (Self, Self) {
		let mut allowed = RolePermission::none();
		let mut denied = RolePermission::none();

		// Emote Permissions
		{
			if value.emote.allow.contains(database::EmotePermission::Upload) {
				allowed |= Self::CreateEmote;
			}
			if value.emote.allow.contains(database::EmotePermission::Edit) {
				allowed |= Self::EditEmote;
			}
			if value.emote.allow.contains(database::EmotePermission::Admin) {
				allowed |= Self::EditAnyEmote;
			}

			if value.emote.deny.contains(database::EmotePermission::Upload) {
				denied |= Self::CreateEmote;
			}
			if value.emote.deny.contains(database::EmotePermission::Edit) {
				denied |= Self::EditEmote;
			}
			if value.emote.deny.contains(database::EmotePermission::Admin) {
				denied |= Self::EditAnyEmote;
			}
		}

		// Role Permissions
		{
			if value.role.allow.contains(database::RolePermission::Admin) {
				allowed |= Self::ManageRoles;
			}
			if value.role.deny.contains(database::RolePermission::Admin) {
				denied |= Self::ManageRoles;
			}
		}

		// Emote Set Permissions
		{
			if value.emote_set.allow.contains(database::EmoteSetPermission::Create) {
				allowed |= Self::CreateEmoteSet;
			}
			if value.emote_set.allow.contains(database::EmoteSetPermission::Edit) {
				allowed |= Self::EditEmoteSet;
			}
			if value.emote_set.allow.contains(database::EmoteSetPermission::Admin) {
				allowed |= Self::EditAnyEmoteSet;
			}

			if value.emote_set.deny.contains(database::EmoteSetPermission::Create) {
				denied |= Self::CreateEmoteSet;
			}
			if value.emote_set.deny.contains(database::EmoteSetPermission::Edit) {
				denied |= Self::EditEmoteSet;
			}
			if value.emote_set.deny.contains(database::EmoteSetPermission::Admin) {
				denied |= Self::EditAnyEmoteSet;
			}
		}

		// Cosmetics Permissions
		{
			if value.badge.allow.contains(database::BadgePermission::Admin)
				&& value.paint.allow.contains(database::PaintPermission::Admin)
			{
				allowed |= Self::ManageCosmetics;
			}

			if value.badge.deny.contains(database::BadgePermission::Admin)
				&& value.paint.deny.contains(database::PaintPermission::Admin)
			{
				denied |= Self::ManageCosmetics;
			}
		}

		// User Permissions
		{
			if value.user.allow.contains(database::UserPermission::Ban) {
				allowed |= Self::ManageBans;
			}
			if value.user.allow.contains(database::UserPermission::Admin) {
				allowed |= Self::ManageUsers;
			}

			if value.user.deny.contains(database::UserPermission::Ban) {
				denied |= Self::ManageBans;
			}
			if value.user.deny.contains(database::UserPermission::Admin) {
				denied |= Self::ManageUsers;
			}
		}

		// Feature Permissions
		{
			if value
				.feature
				.allow
				.contains(database::FeaturePermission::UseCustomProfilePicture)
			{
				allowed |= Self::FeatureProfilePictureAnimation;
			}

			if value
				.feature
				.deny
				.contains(database::FeaturePermission::UseCustomProfilePicture)
			{
				denied |= Self::FeatureProfilePictureAnimation;
			}
		}

		// Report Permissions
		{
			if value.ticket.allow.contains(database::TicketPermission::Create) {
				allowed |= Self::CreateReport;
			}
			if value.ticket.allow.contains(database::TicketPermission::Edit) {
				allowed |= Self::ManageReports;
			}

			if value.ticket.deny.contains(database::TicketPermission::Create) {
				denied |= Self::CreateReport;
			}
			if value.ticket.deny.contains(database::TicketPermission::Edit) {
				denied |= Self::ManageReports;
			}
		}

		// Admin Permissions
		{
			if value.admin.allow.contains(database::AdminPermission::SuperAdmin) {
				allowed |= Self::SuperAdministrator;
			}

			if value.admin.deny.contains(database::AdminPermission::SuperAdmin) {
				denied |= Self::SuperAdministrator;
			}
		}

		(allowed, denied)
	}
}

impl Default for RolePermission {
	fn default() -> Self {
		Self::none()
	}
}

impl Role {
	pub fn from_db(value: database::Role, global_config: &GlobalConfig) -> Self {
		let (allowed, denied) = RolePermission::from_db(value.permissions);

		let position = global_config.role_ids.iter().position(|id| *id == value.id).unwrap_or(0) as u32;

		Self {
			id: value.id.into(),
			name: value.name,
			color: value.color,
			position,
			invisible: false,
			allowed_db: allowed,
			denied_db: denied,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl Role {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.id().timestamp()
	}

	async fn allowed(&self) -> String {
		self.allowed_db.bits().to_string()
	}

	async fn denied(&self) -> String {
		self.denied_db.bits().to_string()
	}

	// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/resolvers/role/role.go#L19
	async fn members(&self, _page: Option<u32>, _limit: Option<u32>) -> Vec<User> {
		// not implemented
		vec![]
	}
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl RolesQuery {
	async fn roles<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Role>, ApiError> {
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let roles = database::Role::collection(global.db())
			.find(doc! {}, None)
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.filter_map(|r| async {
				match r {
					Ok(role) => Some(Role::from_db(role, &global_config)),
					Err(e) => {
						tracing::error!(error = %e, "failed to load role");
						None
					}
				}
			})
			.collect()
			.await;

		Ok(roles)
	}

	async fn role<'ctx>(&self, ctx: &Context<'ctx>, id: RoleObjectId) -> Result<Option<Role>, ApiError> {
		let global = ctx.data::<Arc<Global>>().map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		let global_config = global
			.global_config_loader()
			.load(())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
			.ok_or(ApiError::INTERNAL_SERVER_ERROR)?;

		let role = global
			.role_by_id_loader()
			.load(id.id())
			.await
			.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?;

		Ok(role.map(|r| Role::from_db(r, &global_config)))
	}
}
