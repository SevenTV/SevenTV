use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::image_set::{self, ImageSet, ImageSetInput};
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::editor::{UserEditor, UserEditorId, UserEditorState};
use shared::database::user::profile_picture::UserProfilePicture;
use shared::database::user::settings::UserSettings;
use shared::database::user::{User, UserCached, UserId, UserStyle};

use super::{CdnFileRename, JobOutcome, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub users: &'a mut HashMap<UserId, User>,
	pub profile_pictures: &'a mut Vec<UserProfilePicture>,
	pub editors: &'a mut HashMap<(UserId, UserId), UserEditor>,
	pub entitlements: &'a mut HashSet<EntitlementEdge>,
	pub internal_cdn_rename: &'a mut Vec<CdnFileRename>,
	pub public_cdn_rename: &'a mut Vec<CdnFileRename>,
}

pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("users");

	let RunInput {
		global,
		users,
		profile_pictures,
		editors,
		entitlements,
		internal_cdn_rename,
		public_cdn_rename,
	} = input;

	let mut cursor = global
		.main_source_db
		.collection::<types::User>("users")
		.find(bson::doc! {})
		.await
		.context("query")?;

	let mut all_connections = HashSet::new();

	while let Some(user) = cursor.next().await {
		match user {
			Ok(user) => {
				outcome += process(ProcessInput {
					users,
					profile_pictures,
					editors,
					entitlements,
					user,
					all_connections: &mut all_connections,
					internal_cdn_rename,
					public_cdn_rename,
				});
				outcome.processed_documents += 1;
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	pub users: &'a mut HashMap<UserId, User>,
	pub profile_pictures: &'a mut Vec<UserProfilePicture>,
	pub editors: &'a mut HashMap<(UserId, UserId), UserEditor>,
	pub entitlements: &'a mut HashSet<EntitlementEdge>,
	pub all_connections: &'a mut HashSet<(Platform, String)>,
	pub internal_cdn_rename: &'a mut Vec<CdnFileRename>,
	pub public_cdn_rename: &'a mut Vec<CdnFileRename>,
	pub user: types::User,
}

fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let ProcessInput {
		users,
		profile_pictures,
		editors,
		entitlements,
		mut user,
		all_connections,
		internal_cdn_rename,
		public_cdn_rename,
	} = input;

	if user.connections.is_empty() {
		return ProcessOutcome::default();
	}

	let mut outcome = ProcessOutcome::default();

	let active_profile_picture = match user.avatar {
		Some(types::UserAvatar::Processed {
			input_file, image_files, ..
		}) => {
			let new = match image_set::Image::try_from(input_file.clone()) {
				Ok(input_file) => input_file,
				Err(e) => {
					return outcome.with_error(error::Error::InvalidCdnFile(e));
				}
			};

			let outputs: Vec<_> = match image_files.iter().cloned().map(image_set::Image::try_from).collect() {
				Ok(outputs) => outputs,
				Err(e) => {
					return outcome.with_error(error::Error::InvalidCdnFile(e));
				}
			};

			internal_cdn_rename.push(CdnFileRename {
				old_path: input_file.key.clone(),
				new_path: new.path.clone(),
			});

			for (new, old) in outputs.iter().zip(image_files.iter()) {
				public_cdn_rename.push(CdnFileRename {
					old_path: old.key.clone(),
					new_path: new.path.clone(),
				});
			}

			Some(ImageSet {
				input: ImageSetInput::Image(new),
				outputs,
			})
		}
		Some(types::UserAvatar::Pending { .. }) => None,
		_ => None,
	};

	let profile_picture = active_profile_picture.map(|p| UserProfilePicture {
		id: Default::default(),
		user_id: user.id.into(),
		image_set: p,
		updated_at: chrono::Utc::now(),
	});

	let active_emote_set_id = user
		.connections
		.iter()
		.filter(|c| c.emote_set_id.is_some())
		.min_by(|a, b| a.platform.cmp(&b.platform))
		.map(|c| c.emote_set_id.unwrap().into());

	user.connections.sort_by_key(|c| match c.platform {
		types::ConnectionPlatform::Twitch { .. } => 0,
		types::ConnectionPlatform::Discord { .. } => 1,
		types::ConnectionPlatform::Youtube { .. } => 2,
		types::ConnectionPlatform::Kick { .. } => 3,
	});

	let mut connections = vec![];

	for connection in user.connections {
		let (platform, platform_id, platform_username, platform_display_name, platform_avatar_url) =
			match connection.platform {
				types::ConnectionPlatform::Twitch {
					id: Some(id),
					login,
					display_name,
					profile_image_url,
				} => (Platform::Twitch, id, login, display_name, profile_image_url),
				types::ConnectionPlatform::Discord {
					id: Some(id),
					username,
					avatar,
				} => (
					Platform::Discord,
					id.clone(),
					username.clone(),
					username,
					Some(format!("https://cdn.discordapp.com/avatars/{}/{}.png", id, avatar)),
				),
				types::ConnectionPlatform::Youtube {
					id: Some(id),
					title,
					profile_image_url,
				} => (Platform::Google, id, title.clone(), title, profile_image_url),
				types::ConnectionPlatform::Kick {
					id: Some(id),
					username,
					display_name,
				} => (Platform::Kick, id, username, display_name, None),
				_ => {
					outcome.errors.push(error::Error::MissingPlatformId {
						user_id: user.id,
						platform: connection.platform.into(),
					});
					continue;
				}
			};

		if all_connections.insert((platform, platform_id.clone())) {
			let linked_at = connection.linked_at.into_chrono();

			connections.push(UserConnection {
				platform,
				platform_id,
				platform_username,
				platform_display_name,
				platform_avatar_url,
				updated_at: linked_at,
				linked_at,
				allow_login: true,
			});
		} else {
			outcome
				.errors
				.push(error::Error::DuplicateUserConnection { platform, platform_id });
		}
	}

	users.insert(
		user.id.into(),
		User {
			id: user.id.into(),
			email: user.email,
			email_verified: false,
			settings: UserSettings::default(),
			two_fa: None,
			style: UserStyle {
				active_badge_id: None,
				active_paint_id: None,
				active_emote_set_id,
				active_profile_picture: profile_picture.as_ref().map(|p| p.id),
				pending_profile_picture: None,
				personal_emote_set_id: None,
			},
			connections,
			stripe_customer_id: None,
			paypal_sub_id: None,
			cached: UserCached::default(),
			has_bans: false,
			search_updated_at: None,
			updated_at: chrono::Utc::now(),
		},
	);

	if let Some(profile_picture) = profile_picture {
		profile_pictures.push(profile_picture);
	}

	for editor in user.editors {
		if let Some(editor_id) = editor.id {
			let user_id = user.id.into();
			let editor_id = editor_id.into();

			editors.insert(
				(user_id, editor_id),
				UserEditor {
					id: UserEditorId { user_id, editor_id },
					state: UserEditorState::Accepted,
					notes: None,
					permissions: editor.permissions.to_db(),
					added_by_id: user.id.into(),
					added_at: editor.added_at.into_chrono(),
					search_updated_at: None,
					updated_at: chrono::Utc::now(),
				},
			);
		}
	}

	for role in user.role_ids {
		entitlements.insert(EntitlementEdge {
			id: EntitlementEdgeId {
				from: EntitlementEdgeKind::User { user_id: user.id.into() },
				to: EntitlementEdgeKind::Role { role_id: role.into() },
				managed_by: None,
			},
		});
	}

	outcome
}
