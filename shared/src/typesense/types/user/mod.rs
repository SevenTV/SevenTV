pub mod ban;
pub mod editor;

use std::collections::HashSet;

use chrono::Utc;

use super::TypesenseGenericCollection;
use crate::database;
use crate::database::badge::BadgeId;
use crate::database::emote_set::EmoteSetId;
use crate::database::entitlement::{EntitlementEdgeKind, EntitlementEdgeKindString};
use crate::database::paint::PaintId;
use crate::database::user::UserId;
use crate::typesense::types::TypesenseCollection;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, TypesenseCollection)]
#[typesense(collection_name = "users")]
#[serde(deny_unknown_fields)]
pub struct User {
	pub id: UserId,
	pub discord_names: Vec<String>,
	pub kick_names: Vec<String>,
	pub google_names: Vec<String>,
	pub twitch_names: Vec<String>,
	pub email: Option<String>,
	pub email_verified: bool,
	pub has_bans: bool,
	pub has_2fa: bool,
	pub active_badge_id: Option<BadgeId>,
	pub active_paint_id: Option<PaintId>,
	pub active_emote_set_id: Option<EmoteSetId>,
	pub entitlements: Vec<EntitlementEdgeKindString>,
	pub entitlement_grants: Vec<EntitlementEdgeKindString>,
	pub emotes: Vec<String>,
	#[typesense(default_sort)]
	pub role_rank: i32,
	pub role_hoist_rank: i32,
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl User {
	pub fn from_db(
		value: database::user::User,
		granted_entitlements: impl IntoIterator<Item = EntitlementEdgeKind>,
	) -> Self {
		let (discord_names, kick_names, google_names, twitch_names) = value.connections.into_iter().fold(
			(HashSet::new(), HashSet::new(), HashSet::new(), HashSet::new()),
			|(mut discord_names, mut kick_names, mut google_names, mut twitch_names), connection| {
				match connection.platform {
					database::user::connection::Platform::Discord => &mut discord_names,
					database::user::connection::Platform::Kick => &mut kick_names,
					database::user::connection::Platform::Google => &mut google_names,
					database::user::connection::Platform::Twitch => &mut twitch_names,
				}
				.extend([
					connection.platform_username,
					connection.platform_id,
					connection.platform_display_name,
				]);

				(discord_names, kick_names, google_names, twitch_names)
			},
		);

		Self {
			id: value.id,
			email: value.email,
			email_verified: value.email_verified,
			discord_names: discord_names.into_iter().collect(),
			kick_names: kick_names.into_iter().collect(),
			google_names: google_names.into_iter().collect(),
			twitch_names: twitch_names.into_iter().collect(),
			has_bans: value.has_bans,
			has_2fa: value.two_fa.is_some(),
			active_badge_id: value.style.active_badge_id.and_then(|id| {
				value
					.cached
					.entitlements
					.contains(&EntitlementEdgeKind::Badge { badge_id: id })
					.then_some(id)
			}),
			active_paint_id: value.style.active_paint_id.and_then(|id| {
				value
					.cached
					.entitlements
					.contains(&EntitlementEdgeKind::Paint { paint_id: id })
					.then_some(id)
			}),
			active_emote_set_id: value.style.active_emote_set_id,
			entitlement_grants: granted_entitlements
				.into_iter()
				.map(Into::into)
				.collect::<HashSet<_>>()
				.into_iter()
				.collect(),
			entitlements: value
				.cached
				.entitlements
				.into_iter()
				.map(Into::into)
				.collect::<HashSet<_>>()
				.into_iter()
				.collect(),
			emotes: value.cached.active_emotes.into_iter().map(|id| id.to_string()).collect(),
			role_rank: value.cached.role_rank,
			role_hoist_rank: value.cached.role_hoist_rank,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	std::iter::once(TypesenseGenericCollection::new::<User>())
		.chain(ban::typesense_collections())
		.chain(editor::typesense_collections())
}
