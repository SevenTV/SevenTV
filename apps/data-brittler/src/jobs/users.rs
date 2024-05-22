use std::sync::Arc;

use anyhow::Context;
use fnv::{FnvHashMap, FnvHashSet};
use futures::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::options::InsertManyOptions;
use shared::database::{
	Collection, ImageSet, ImageSetInput, Platform, RoleId, User, UserConnection, UserConnectionId, UserEditor, UserEditorId, UserEditorPermissions, UserEditorState, UserEntitledCache, UserGrants, UserSettings, UserStyle
};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::{error, types};

pub struct UsersJob {
	global: Arc<Global>,
	entitlements: FnvHashMap<ObjectId, Vec<types::Entitlement>>,
	all_connections: FnvHashSet<(Platform, String)>,
	users: Vec<User>,
	connections: Vec<UserConnection>,
	editors: Vec<UserEditor>,
}

impl Job for UsersJob {
	type T = types::User;

	const NAME: &'static str = "transfer_users";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping users and user_connections collections");
			User::collection(global.target_db()).drop(None).await?;
			UserConnection::collection(global.target_db()).drop(None).await?;
			UserEditor::collection(global.target_db()).drop(None).await?;
		}

		tracing::info!("querying all entitlements");
		let mut entitlements_cursor = global
			.source_db()
			.collection::<types::Entitlement>("entitlements")
			.find(None, None)
			.await?;
		let mut entitlements: FnvHashMap<ObjectId, Vec<types::Entitlement>> = FnvHashMap::default();
		while let Some(entitlement) = entitlements_cursor
			.try_next()
			.await
			.context("failed to deserialize entitlement")?
		{
			// Ignore all entitlements without a user_id
			if let Some(user_id) = entitlement.user_id {
				entitlements.entry(user_id).or_default().push(entitlement);
			}
		}

		Ok(Self {
			global,
			entitlements,
			all_connections: FnvHashSet::default(),
			users: vec![],
			connections: vec![],
			editors: vec![],
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("users")
	}

	async fn process(&mut self, user: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let entitlements = self.entitlements.remove(&user.id).unwrap_or_default();

		let mut roles = FnvHashSet::default();

		for role_id in entitlements.iter().filter_map(|e| match &e.data {
			types::EntitlementData::Role { ref_id } => Some(ref_id),
			_ => None,
		}) {
			roles.insert(RoleId::from(*role_id));
		}

		let active_badge_id = entitlements
			.iter()
			.find(|e| matches!(e.data, types::EntitlementData::Badge { selected: true, .. }))
			.map(|e| e.id);

		let active_paint_id = entitlements
			.iter()
			.find(|e| matches!(e.data, types::EntitlementData::Paint { selected: true, .. }))
			.map(|e| e.id);

		let active_profile_picture = match user.avatar {
			Some(types::UserAvatar::Processed {
				input_file, image_files, ..
			}) => Some(ImageSet {
				input: ImageSetInput::Image(input_file.into()),
				outputs: image_files.into_iter().map(Into::into).collect(),
			}),
			// Some(types::UserAvatar::Pending { pending_id }) => Some(ImageSet {
			// 	input: ImageSetInput::Pending { path: todo!(), mime: todo!(), size: todo!() },
			// 	outputs: vec![],
			// }),
			Some(types::UserAvatar::Pending { .. }) => {
				outcome.errors.push(error::Error::NotImplemented("pending avatar"));
				None
			}
			_ => None,
		};

		self.users.push(User {
			id: user.id.into(),
			email: user.email,
			email_verified: false,
			password_hash: None,
			settings: UserSettings::default(),
			two_fa: None,
			style: UserStyle {
				active_badge_id: active_badge_id.map(Into::into),
				active_paint_id: active_paint_id.map(Into::into),
				active_profile_picture: active_profile_picture.clone(),
				all_profile_pictures: active_profile_picture.map(|p| vec![p]).unwrap_or_default(),
			},
			active_emote_set_ids: vec![],
			grants: UserGrants {
				role_ids: roles.into_iter().collect(),
				..Default::default()
			},
			entitled_cache: UserEntitledCache::default(),
		});

		for (i, connection) in user.connections.into_iter().enumerate() {
			let id = UserConnectionId::with_timestamp(connection.linked_at.into_chrono());

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

			if self.all_connections.insert((platform, platform_id.clone())) {
				self.connections.push(UserConnection {
					id,
					user_id: user.id.into(),
					main_connection: i == 0,
					platform,
					platform_id,
					platform_username,
					platform_display_name,
					platform_avatar_url,
					allow_login: true,
				});
			} else {
				outcome
					.errors
					.push(error::Error::DuplicateUserConnection { platform, platform_id });
			}
		}

		for editor in user.editors {
			if let Some(editor_id) = editor.id {
				let permissions = UserEditorPermissions {};

				self.editors.push(UserEditor {
					id: UserEditorId::new(),
					user_id: user.id.into(),
					editor_id: editor_id.into(),
					state: UserEditorState::Accepted,
					notes: None,
					permissions,
					added_by_id: Some(user.id.into()),
				});
			}
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing users job");

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let users = User::collection(self.global.target_db());
		let connections = UserConnection::collection(self.global.target_db());
		let editors = UserEditor::collection(self.global.target_db());

		let res = tokio::join!(
			users.insert_many(&self.users, insert_options.clone()),
			connections.insert_many(&self.connections, insert_options.clone()),
			editors.insert_many(&self.editors, insert_options),
		);
		let res =
			vec![res.0, res.1, res.2]
				.into_iter()
				.zip(vec![self.users.len(), self.connections.len(), self.editors.len()]);

		for (res, len) in res {
			match res {
				Ok(res) => {
					outcome.inserted_rows += res.inserted_ids.len() as u64;
					if res.inserted_ids.len() != len {
						outcome.errors.push(error::Error::InsertMany);
					}
				}
				Err(e) => outcome.errors.push(e.into()),
			}
		}

		outcome
	}
}
