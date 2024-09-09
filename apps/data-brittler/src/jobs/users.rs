use std::future::IntoFuture;
use std::sync::Arc;

use anyhow::Context;
use fnv::{FnvHashMap, FnvHashSet};
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::InsertManyOptions;
use shared::database::entitlement::{EntitlementEdge, EntitlementEdgeId, EntitlementEdgeKind};
use shared::database::image_set::{self, ImageSet, ImageSetInput};
use shared::database::user::connection::{Platform, UserConnection};
use shared::database::user::editor::{UserEditor, UserEditorId, UserEditorState};
use shared::database::user::profile_picture::UserProfilePicture;
use shared::database::user::settings::UserSettings;
use shared::database::user::{User, UserCached, UserId, UserStyle};
use shared::database::MongoCollection;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types::EntitlementData;
use crate::{error, types};

pub struct UsersJob {
	global: Arc<Global>,
	entitlements: FnvHashMap<ObjectId, Vec<types::Entitlement>>,
	paypal_subs: FnvHashMap<ObjectId, String>,
	all_connections: FnvHashSet<(Platform, String)>,
	users: Vec<User>,
	profile_pictures: Vec<UserProfilePicture>,
	editors: FnvHashMap<(UserId, UserId), UserEditor>,
	edges: FnvHashSet<EntitlementEdge>,
}

impl Job for UsersJob {
	type T = types::User;

	const NAME: &'static str = "transfer_users";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("dropping users and user_editors collections");

			User::collection(global.target_db()).drop().await?;
			let indexes = User::indexes();
			if !indexes.is_empty() {
				User::collection(global.target_db()).create_indexes(indexes).await?;
			}

			UserEditor::collection(global.target_db()).drop().await?;
			let indexes = UserEditor::indexes();
			if !indexes.is_empty() {
				UserEditor::collection(global.target_db()).create_indexes(indexes).await?;
			}
		}

		let mut paypal_subs = FnvHashMap::default();

		let mut subs_cursor = global
			.egvault_source_db()
			.collection::<types::Subscription>("subscriptions")
			.find(doc! {
				"provider": "paypal"
			})
			.await?;

		while let Some(sub) = subs_cursor.try_next().await.context("failed to deserialize sub")? {
			if sub.provider == types::SubscriptionProvider::Paypal {
				if let Some(provider_id) = sub.provider_id {
					paypal_subs.insert(sub.customer_id, provider_id);
				}
			}
		}

		let mut edges = FnvHashSet::default();

		tracing::info!("querying all entitlements");
		let mut entitlements_cursor = global
			.source_db()
			.collection::<types::Entitlement>("entitlements")
			.find(doc! {})
			.await?;
		let mut entitlements: FnvHashMap<ObjectId, Vec<types::Entitlement>> = FnvHashMap::default();
		while let Some(entitlement) = entitlements_cursor
			.try_next()
			.await
			.context("failed to deserialize entitlement")?
		{
			// Ignore all entitlements without a user_id
			if let Some(user_id) = entitlement.user_id {
				if let EntitlementData::Role { ref_id } = entitlement.data {
					// Ignore the `Subscriber` role because it is handled by the subscription job.
					if ref_id.to_string() == "6076a86b09a4c63a38ebe801" {
						continue;
					}

					edges.insert(EntitlementEdge {
						id: EntitlementEdgeId {
							from: EntitlementEdgeKind::User { user_id: user_id.into() },
							to: EntitlementEdgeKind::Role { role_id: ref_id.into() },
							managed_by: None,
						},
					});
				}

				entitlements.entry(user_id).or_default().push(entitlement);
			}
		}

		Ok(Self {
			global,
			entitlements,
			paypal_subs,
			profile_pictures: vec![],
			all_connections: FnvHashSet::default(),
			users: vec![],
			editors: FnvHashMap::default(),
			edges,
		})
	}

	async fn collection(&self) -> Option<mongodb::Collection<Self::T>> {
		Some(self.global.source_db().collection("users"))
	}

	async fn process(&mut self, mut user: Self::T) -> ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let entitlements = self.entitlements.remove(&user.id).unwrap_or_default();

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
			}) => {
				let input_file = match image_set::Image::try_from(input_file) {
					Ok(input_file) => input_file,
					Err(e) => {
						return outcome.with_error(error::Error::InvalidCdnFile(e));
					}
				};

				let outputs = match image_files.into_iter().map(image_set::Image::try_from).collect() {
					Ok(outputs) => outputs,
					Err(e) => {
						return outcome.with_error(error::Error::InvalidCdnFile(e));
					}
				};

				Some(ImageSet {
					input: ImageSetInput::Image(input_file),
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

		user.connections.sort_by_key(|c| {
			match c.platform {
				types::ConnectionPlatform::Twitch { .. } => 0,
				types::ConnectionPlatform::Discord { .. } => 1,
				types::ConnectionPlatform::Youtube { .. } => 2,
				types::ConnectionPlatform::Kick { .. } => 3,
			}
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

			if self.all_connections.insert((platform, platform_id.clone())) {
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

		let paypal_sub_id = self.paypal_subs.remove(&user.id);

		self.users.push(User {
			id: user.id.into(),
			email: user.email,
			email_verified: false,
			settings: UserSettings::default(),
			two_fa: None,
			style: UserStyle {
				active_badge_id: active_badge_id.map(Into::into),
				active_paint_id: active_paint_id.map(Into::into),
				active_emote_set_id,
				active_profile_picture: profile_picture.as_ref().map(|p| p.id),
				pending_profile_picture: None,
			},
			connections,
			stripe_customer_id: None,
			paypal_sub_id,
			cached: UserCached::default(),
			has_bans: false,
			search_updated_at: None,
			updated_at: chrono::Utc::now(),
		});

		if let Some(profile_picture) = profile_picture {
			self.profile_pictures.push(profile_picture);
		}

		for editor in user.editors {
			if let Some(editor_id) = editor.id {
				let user_id = user.id.into();
				let editor_id = editor_id.into();

				self.editors.insert(
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
			self.edges.insert(EntitlementEdge {
				id: EntitlementEdgeId {
					from: EntitlementEdgeKind::User { user_id: user.id.into() },
					to: EntitlementEdgeKind::Role { role_id: role.into() },
					managed_by: None,
				},
			});
		}

		outcome
	}

	async fn finish(self) -> ProcessOutcome {
		tracing::info!("finishing users job");

		// In case of truncate = true, we have to wait for the entitlements job to
		// finish truncating. Otherwise we will loose the edges here.
		if self.global.config().should_run_entitlements() && self.global.config().truncate {
			self.global.entitlement_job_token().cancelled().await;
		}

		let mut outcome = ProcessOutcome::default();

		let insert_options = InsertManyOptions::builder().ordered(false).build();
		let users = User::collection(self.global.target_db());
		let editors = UserEditor::collection(self.global.target_db());
		let edges = EntitlementEdge::collection(self.global.target_db());
		let profile_pictures = UserProfilePicture::collection(self.global.target_db());

		let res = tokio::join!(
			users
				.insert_many(&self.users)
				.with_options(insert_options.clone())
				.into_future(),
			editors
				.insert_many(self.editors.values())
				.with_options(insert_options.clone())
				.into_future(),
			edges
				.insert_many(&self.edges)
				.with_options(insert_options.clone())
				.into_future(),
			profile_pictures
				.insert_many(&self.profile_pictures)
				.with_options(insert_options.clone())
				.into_future(),
		);
		let res = vec![res.0, res.1, res.2, res.3].into_iter().zip(vec![
			self.users.len(),
			self.editors.len(),
			self.edges.len(),
			self.profile_pictures.len(),
		]);

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

		self.global.users_job_token().cancel();

		outcome
	}
}
