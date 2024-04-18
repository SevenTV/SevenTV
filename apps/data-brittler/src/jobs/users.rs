use std::sync::Arc;

use anyhow::Context;
use fnv::{FnvHashMap, FnvHashSet};
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use shared::database::{
	self, Collection, FileSet, FileSetProperties, Platform, User, UserConnection, UserEntitledCache, UserGrants,
	UserSettings, UserStyle,
};

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::image_files_to_file_properties;
use crate::{error, types};

// TODO: editors

pub struct UsersJob {
	global: Arc<Global>,
	entitlements: FnvHashMap<ObjectId, Vec<types::Entitlement>>,
	all_connections: FnvHashSet<(Platform, String)>,
}

impl Job for UsersJob {
	type T = types::User;

	const NAME: &'static str = "transfer_users";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating users, user_connections and user_roles collections");
			User::collection(global.target_db()).delete_many(doc! {}, None);
			UserConnection::collection(global.target_db()).delete_many(doc! {}, None);

			tracing::info!("deleting profile picture files from file_sets collection");
			FileSet::collection(global.target_db())
				.delete_many(
					doc! { "kind": mongodb::bson::to_bson(&database::FileSetKind::ProfilePicture)? },
					None,
				)
				.await?;
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
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.source_db().collection("users")
	}

	async fn process(&mut self, user: Self::T) -> ProcessOutcome {
		let entitlements = self.entitlements.remove(&user.id).unwrap_or_default();

		let mut roles = FnvHashSet::default();

		for role_id in entitlements.iter().filter_map(|e| match &e.data {
			types::EntitlementData::Role { ref_id } => Some(ref_id),
			_ => None,
		}) {
			roles.insert(role_id.clone());
		}

		let active_badge_id = entitlements
			.iter()
			.find(|e| matches!(e.data, types::EntitlementData::Badge { selected: true, .. }))
			.map(|e| e.id);

		let active_paint_id = entitlements
			.iter()
			.find(|e| matches!(e.data, types::EntitlementData::Paint { selected: true, .. }))
			.map(|e| e.id);

		let (pending_profile_picture_id, active_profile_picture_id) = match user.avatar {
			Some(types::UserAvatar::Processed {
				id,
				input_file,
				image_files,
			}) => {
				let outputs = match image_files_to_file_properties(image_files) {
					Ok(outputs) => outputs,
					Err(e) => {
						return ProcessOutcome {
							errors: vec![e.into()],
							inserted_rows: 0,
						};
					}
				};

				if let Err(e) = FileSet::collection(self.global.target_db())
					.insert_one(
						FileSet {
							id,
							kind: database::FileSetKind::ProfilePicture,
							authenticated: false,
							properties: FileSetProperties::Image {
								input: input_file.into(),
								pending: false,
								outputs,
							},
						},
						None,
					)
					.await
				{
					return ProcessOutcome {
						errors: vec![e.into()],
						inserted_rows: 0,
					};
				}

				(None, Some(id))
			}
			Some(types::UserAvatar::Pending { pending_id }) => (Some(pending_id), None),
			_ => (None, None),
		};

		if let Err(e) = User::collection(self.global.target_db())
			.insert_one(
				User {
					id: user.id,
					email: user.email,
					email_verified: false,
					password_hash: None,
					settings: UserSettings::default(),
					two_fa: None,
					style: UserStyle {
						active_badge_id,
						active_paint_id,
						pending_profile_picture_id,
						active_profile_picture_id,
						all_profile_picture_ids: active_profile_picture_id.map(|id| vec![id]).unwrap_or_default(),
					},
					active_emote_set_ids: vec![],
					grants: UserGrants {
						role_ids: roles.into_iter().collect(),
						..Default::default()
					},
					entitled_cache: UserEntitledCache::default(),
				},
				None,
			)
			.await
		{
			return ProcessOutcome {
				errors: vec![e.into()],
				inserted_rows: 0,
			};
		}

		let mut errors = Vec::new();
		let mut inserted_rows = 1;

		for (i, connection) in user.connections.into_iter().enumerate() {
			let id = crate::database::object_id_from_datetime(connection.linked_at.into_chrono());

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
						errors.push(error::Error::MissingPlatformId {
							user_id: user.id,
							platform: connection.platform.into(),
						});
						continue;
					}
				};

			if self.all_connections.insert((platform, platform_id.clone())) {
				match UserConnection::collection(self.global.target_db()).insert_one(UserConnection {
					id,
					user_id: user.id,
					main_connection: i == 0,
					platform,
					platform_id,
					platform_username,
					platform_display_name,
					platform_avatar_url,
					allow_login: true,
				}, None).await
				{
					Ok(_) => inserted_rows += 1,
					Err(e) => errors.push(e.into()),
				}
			} else {
				errors.push(error::Error::DuplicateUserConnection { platform, platform_id });
			}
		}

		ProcessOutcome { errors, inserted_rows }
	}

	async fn finish(self) -> anyhow::Result<()> {
		tracing::info!("finishing users job");

		Ok(())
	}
}
