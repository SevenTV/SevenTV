use std::collections::HashMap;

use shared::database::role::permissions::{
	AdminPermission, EmoteModerationRequestPermission, EmotePermission, EmoteSetPermission, PaintPermission, Permission,
	Permissions, RateLimitResource, RateLimits, RolePermission, TicketPermission, UserPermission,
};
use shared::database::role::{Role, RoleId};
use shared::database::user::UserId;

#[derive(Default)]
struct RoleBuilder<'a> {
	name: &'a str,
	id: &'a str,
	rank: i32,
	color: Option<i32>,
	hoist: bool,

	emote_set_capacity: Option<i32>,
	personal_emote_set_capacity: Option<i32>,
	emote_moderation_request_priority: Option<i32>,
	emote_moderation_request_limit: Option<i32>,
	ratelimits: HashMap<RateLimitResource, Option<RateLimits>>,
	allowed: &'a [Permission],
	denied: &'a [Permission],
}

impl RoleBuilder<'_> {
	fn build(self) -> Role {
		Role {
			id: self.id.parse().unwrap(),
			name: self.name.to_string(),
			color: self.color,
			rank: self.rank,
			hoist: self.hoist,
			updated_at: chrono::Utc::now(),
			created_by: UserId::nil(),
			tags: vec![],
			description: None,
			applied_rank: None,
			search_updated_at: None,
			permissions: {
				let mut perms = Permissions::default();
				for perm in self.allowed {
					perms.allow(*perm);
				}
				for perm in self.denied {
					perms.deny(*perm);
				}

				perms.emote_set_capacity = self.emote_set_capacity.map(Into::into);
				perms.personal_emote_set_capacity = self.personal_emote_set_capacity.map(Into::into);
				perms.emote_moderation_request_priority = self.emote_moderation_request_priority.map(Into::into);
				perms.emote_moderation_request_limit = self.emote_moderation_request_limit.map(Into::into);

				for (resource, limits) in self.ratelimits {
					perms.ratelimits.insert(resource.as_str().to_owned(), limits);
				}

				perms
			},
		}
	}
}

pub fn roles() -> Vec<Role> {
	[
		RoleBuilder {
			name: "Superuser",
			id: "630384e764dbf4a6d1b3f7c7",
			rank: 1000,
			allowed: &[AdminPermission::SuperAdmin.into()],
			..Default::default()
		},
		RoleBuilder {
			name: "Staff",
			id: "63124dcf098bd6b8e5a7cb02",
			rank: 900,
			..Default::default()
		},
		RoleBuilder {
			name: "Dungeon Mistress",
			id: "608831312a61f51b61f2974d",
			rank: 800,
			color: Some(-8328449),
			hoist: true,
			..Default::default()
		},
		RoleBuilder {
			name: "Admin",
			id: "6102002eab1aa12bf648cfcd",
			rank: 700,
			color: Some(-12171521),
			hoist: true,
			allowed: &[AdminPermission::Admin.into()],
			..Default::default()
		},
		RoleBuilder {
			name: "Event Coordinator",
			id: "631ef5ea03e9beb96f849a7e",
			rank: 600,
			color: Some(-1237797377),
			hoist: true,
			..Default::default()
		},
		RoleBuilder {
			name: "Moderator",
			id: "60724f65e93d828bf8858789",
			rank: 500,
			color: Some(849892095),
			hoist: true,
			allowed: &[
				EmotePermission::ManageAny.into(),
				EmotePermission::ViewUnlisted.into(),
				EmotePermission::Merge.into(),
				EmoteSetPermission::ManageAny.into(),
				TicketPermission::ManageAbuse.into(),
				TicketPermission::ManageGeneric.into(),
				EmoteModerationRequestPermission::Manage.into(),
				UserPermission::ManageAny.into(),
				UserPermission::Moderate.into(),
				UserPermission::ViewHidden.into(),
				RolePermission::Assign.into(),
			],
			..Default::default()
		},
		RoleBuilder {
			name: "Trial Moderator",
			id: "612c888812a39cc5cdd82ae0",
			rank: 450,
			color: Some(1153599487),
			hoist: true,
			allowed: &[
				EmotePermission::ManageAny.into(),
				EmotePermission::ViewUnlisted.into(),
				TicketPermission::ManageAbuse.into(),
				TicketPermission::ManageGeneric.into(),
				EmoteModerationRequestPermission::Manage.into(),
				UserPermission::ViewHidden.into(),
			],
			..Default::default()
		},
		RoleBuilder {
			name: "Contributor",
			id: "60b3f1ea886e63449c5263b1",
			rank: 400,
			color: Some(401323775),
			hoist: true,
			..Default::default()
		},
		RoleBuilder {
			name: "Translator",
			id: "62f99d0ce46eb00e438a6984",
			rank: 350,
			color: Some(-1309985537),
			hoist: true,
			..Default::default()
		},
		RoleBuilder {
			name: "Painter",
			id: "62d86a8419fdcf401421c5ae",
			rank: 300,
			allowed: &[PaintPermission::Manage.into(), PaintPermission::Assign.into()],
			..Default::default()
		},
		RoleBuilder {
			name: "Entitler",
			id: "65786434230d123dfb557550",
			rank: 250,
			..Default::default()
		},
		RoleBuilder {
			name: "Upload Restricted",
			id: "632df48b3999124c6544d7c4",
			rank: 200,
			denied: &[EmotePermission::Upload.into()],
			..Default::default()
		},
		RoleBuilder {
			name: "Reports Restricted",
			id: "634cea6e87766b247bcb6d90",
			rank: 190,
			denied: &[TicketPermission::Create.into(), TicketPermission::Message.into()],
			..Default::default()
		},
		RoleBuilder {
			name: "NoPerms",
			id: "63725621df2cfba2d34a46fb",
			rank: 180,
			hoist: true,
			denied: &[
				UserPermission::UseCustomProfilePicture.into(),
				UserPermission::UsePersonalEmoteSet.into(),
				UserPermission::UseBadge.into(),
				UserPermission::UsePaint.into(),
				UserPermission::Login.into(),
				UserPermission::InviteEditors.into(),
				EmotePermission::Upload.into(),
				EmotePermission::Edit.into(),
				EmotePermission::Delete.into(),
				EmoteSetPermission::Manage.into(),
				EmoteSetPermission::Resize.into(),
				TicketPermission::Create.into(),
				TicketPermission::Message.into(),
			],
			..Default::default()
		},
		RoleBuilder {
			name: "Verified",
			id: "6076a99409a4c63a38ebe802",
			rank: 150,
			color: Some(-1857617921),
			hoist: true,
			..Default::default()
		},
		RoleBuilder {
			name: "Subscriber",
			id: "6076a86b09a4c63a38ebe801",
			rank: 100,
			color: Some(-5635841),
			hoist: true,
			allowed: &[
				UserPermission::UseCustomProfilePicture.into(),
				UserPermission::UsePersonalEmoteSet.into(),
			],
			..Default::default()
		},
		RoleBuilder {
			name: "Verified Bot",
			id: &RoleId::new().to_string(),
			rank: 50,
			ratelimits: HashMap::from([
				(
					RateLimitResource::Global,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 20_000,
						overuse_threshold: Some(25_000),
						overuse_punishment: Some(300),
					}),
				),
				(
					RateLimitResource::Search,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 1_000,
						overuse_threshold: Some(1250),
						overuse_punishment: Some(300),
					}),
				),
				(
					RateLimitResource::EmoteSetChange,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 5_000,
						overuse_threshold: Some(7_500),
						overuse_punishment: Some(300),
					}),
				),
				(
					RateLimitResource::EmoteSetCreate,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 500,
						overuse_threshold: Some(800),
						overuse_punishment: Some(300),
					}),
				),
			]),
			..Default::default()
		},
		RoleBuilder {
			name: "Default",
			id: "62b48deb791a15a25c2a0354",
			rank: 1,
			allowed: &[
				UserPermission::Billing.into(),
				UserPermission::UseBadge.into(),
				UserPermission::UsePaint.into(),
				UserPermission::Login.into(),
				UserPermission::InviteEditors.into(),
				EmotePermission::Upload.into(),
				EmotePermission::Edit.into(),
				EmotePermission::Delete.into(),
				EmoteSetPermission::Manage.into(),
				EmoteSetPermission::Resize.into(),
				TicketPermission::Create.into(),
				TicketPermission::Message.into(),
			],
			emote_set_capacity: Some(1000),
			emote_moderation_request_priority: Some(1),
			emote_moderation_request_limit: Some(5),
			ratelimits: HashMap::from([
				(
					RateLimitResource::EmoteUpload,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 5,
						overuse_threshold: Some(50),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::ProfilePictureUpload,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 5,
						overuse_threshold: Some(50),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::Login,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 10,
						overuse_threshold: Some(20),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::Search,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 100,
						overuse_threshold: Some(500),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::UserChangeCosmetics,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 10,
						overuse_threshold: Some(15),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::UserChangeEditor,
					Some(RateLimits {
						interval_seconds: 1800,
						requests: 30,
						overuse_threshold: Some(60),
						overuse_punishment: Some(3600 * 6),
					}),
				),
				(
					RateLimitResource::UserChangeConnections,
					Some(RateLimits {
						interval_seconds: 1800,
						requests: 30,
						overuse_threshold: Some(60),
						overuse_punishment: Some(3600 * 6),
					}),
				),
				(
					RateLimitResource::EmoteUpdate,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 30,
						overuse_threshold: None,
						overuse_punishment: None,
					}),
				),
				(
					RateLimitResource::EmoteSetCreate,
					Some(RateLimits {
						interval_seconds: 1800,
						requests: 10,
						overuse_threshold: Some(20),
						overuse_punishment: Some(3600 * 6),
					}),
				),
				(
					RateLimitResource::EmoteSetChange,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 100,
						overuse_threshold: Some(500),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::EgVaultSubscribe,
					Some(RateLimits {
						interval_seconds: 1800,
						requests: 10,
						overuse_threshold: Some(50),
						overuse_punishment: Some(3600),
					}),
				),
				(
					RateLimitResource::EgVaultRedeem,
					Some(RateLimits {
						interval_seconds: 300,
						requests: 20,
						overuse_threshold: Some(50),
						overuse_punishment: Some(3600 * 12),
					}),
				),
				(
					RateLimitResource::EgVaultPaymentMethod,
					Some(RateLimits {
						interval_seconds: 300,
						requests: 20,
						overuse_threshold: Some(50),
						overuse_punishment: Some(3600 * 12),
					}),
				),
				(
					RateLimitResource::Global,
					Some(RateLimits {
						interval_seconds: 60,
						requests: 5_000,
						overuse_threshold: Some(7_500),
						overuse_punishment: Some(1800),
					}),
				),
			]),
			..Default::default()
		},
	]
	.into_iter()
	.map(|b| b.build())
	.collect()
}
