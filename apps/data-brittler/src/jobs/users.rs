use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use scuffle_image_processor_proto::EventCallback;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::image_set::{self, ImageSet, ImageSetInput};
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::editor::{UserEditor, UserEditorId, UserEditorState};
use shared::database::user::profile_picture::{UserProfilePicture, UserProfilePictureId};
use shared::database::user::settings::UserSettings;
use shared::database::user::{User, UserCached, UserId, UserStyle};

use super::cosmetics::PendingTask;
use super::{CdnFileRename, JobOutcome, ProcessOutcome};
use crate::download_cosmetics::request_image;
use crate::global::Global;
use crate::{error, types};

pub struct RunInput<'a> {
	pub global: &'a Arc<Global>,
	pub users: &'a mut HashMap<UserId, User>,
	pub pending_tasks: &'a mut Vec<(PendingTask, tokio::sync::mpsc::Receiver<EventCallback>)>,
	pub profile_pictures: &'a mut HashMap<UserProfilePictureId, UserProfilePicture>,
	pub editors: &'a mut HashMap<(UserId, UserId), UserEditor>,
	pub entitlements: &'a mut HashSet<EntitlementEdge>,
	pub internal_cdn_rename: &'a mut Vec<CdnFileRename>,
	pub public_cdn_rename: &'a mut Vec<CdnFileRename>,
}

#[tracing::instrument(skip_all, name = "users")]
pub async fn run(input: RunInput<'_>) -> anyhow::Result<JobOutcome> {
	let mut outcome = JobOutcome::new("users");

	let RunInput {
		global,
		pending_tasks,
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
					global,
					users,
					profile_pictures,
					editors,
					entitlements,
					user,
					pending_tasks,
					all_connections: &mut all_connections,
					internal_cdn_rename,
					public_cdn_rename,
				})
				.await;
				outcome.processed_documents += 1;
			}
			Err(e) => {
				tracing::error!("{:#}", e);
				outcome.errors.push(e.into());
			}
		}
	}

	Ok(outcome)
}

struct ProcessInput<'a> {
	pub global: &'a Arc<Global>,
	pub users: &'a mut HashMap<UserId, User>,
	pub pending_tasks: &'a mut Vec<(PendingTask, tokio::sync::mpsc::Receiver<EventCallback>)>,
	pub profile_pictures: &'a mut HashMap<UserProfilePictureId, UserProfilePicture>,
	pub editors: &'a mut HashMap<(UserId, UserId), UserEditor>,
	pub entitlements: &'a mut HashSet<EntitlementEdge>,
	pub all_connections: &'a mut HashSet<(Platform, String)>,
	pub internal_cdn_rename: &'a mut Vec<CdnFileRename>,
	pub public_cdn_rename: &'a mut Vec<CdnFileRename>,
	pub user: types::User,
}

async fn process(input: ProcessInput<'_>) -> ProcessOutcome {
	let ProcessInput {
		global,
		pending_tasks,
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

	let ip = &global.image_processor;

	let mut new_pfp_id = UserProfilePictureId::new();

	user.avatar_id = if let Some("") = user.avatar_id.as_deref() {
		None
	} else {
		user.avatar_id
	};

	let active_profile_picture = match (user.avatar, user.avatar_id) {
		(
			Some(types::UserAvatar::Processed {
				id,
				input_file, image_files, ..
			}),
			_,
		) => {
			new_pfp_id = id.into();

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
		(Some(types::UserAvatar::Pending { .. }), _) => None,
		(_, Some(pfp_id)) if global.config.should_run_cosmetics() => {
			new_pfp_id = pfp_id.parse().unwrap();

			let image_data = match tokio::fs::read(format!("local/cosmetics/{}:{}", user.id, pfp_id)).await {
				Ok(data) => Some(bytes::Bytes::from(data)),
				Err(e) => {
					if let std::io::ErrorKind::NotFound = e.kind() {
						let download_url = format!("https://cdn.7tv.app/pp/{}/{}", user.id, pfp_id);
						match request_image(global, &download_url).await {
							Ok(data) => Some(data),
							Err(err) => {
								tracing::error!(error = ?err, "failed to download image: {download_url}");
								None
							},
						}
					} else {
						tracing::error!(error = ?e, "failed to read image from disk");
						None
					}
				}
			};

			let input = if let Some(image_data) = image_data {
				match ip.upload_profile_picture(new_pfp_id, user.id.into(), image_data, None).await {
					Ok(scuffle_image_processor_proto::ProcessImageResponse { error: Some(error), .. }) => {
						tracing::error!(error = ?error, "failed to start processing image");
						None
					}
					Ok(scuffle_image_processor_proto::ProcessImageResponse {
						id,
						upload_info:
							Some(scuffle_image_processor_proto::ProcessImageResponseUploadInfo {
								path: Some(path),
								content_type,
								size,
							}),
						error: None,
					}) => {
						let (tx, rx) = tokio::sync::mpsc::channel(10);
						pending_tasks.push((PendingTask::UserProfilePicture(new_pfp_id), rx));
						global.all_tasks.lock().await.insert(id.clone(), tx);
						tracing::info!(task_id = %id, "started send image processor request");
						Some(ImageSetInput::Pending {
							task_id: id,
							path: path.path,
							mime: content_type,
							size: size as i64,
						})
					}
					Err(err) => {
						tracing::error!(error = ?err, "failed to start send image processor request");
						None
					}
					_ => None,
				}
			} else {
				None
			};

			input.map(|input| ImageSet { input, outputs: vec![] })
		}
		_ => None,
	};

	let profile_picture = active_profile_picture.map(|p| UserProfilePicture {
		id: new_pfp_id,
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
		profile_pictures.insert(profile_picture.id, profile_picture);
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
