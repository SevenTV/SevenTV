use std::pin::Pin;
use std::sync::Arc;

use anyhow::Context;
use fnv::{FnvHashMap, FnvHashSet};
use futures::TryStreamExt;
use postgres_from_row::tokio_postgres::binary_copy::BinaryCopyInWriter;
use postgres_types::Type;
use shared::database::Platform;
use shared::object_id::ObjectId;

use super::{Job, ProcessOutcome};
use crate::database::{file_set_kind_type, platform_enum_type};
use crate::global::Global;
use crate::types::image_files_to_file_properties;
use crate::{error, types};

pub struct UsersJob {
	global: Arc<Global>,
	entitlements: FnvHashMap<ObjectId, Vec<types::Entitlement>>,
	users_writer: Pin<Box<BinaryCopyInWriter>>,
	user_roles_writer: Pin<Box<BinaryCopyInWriter>>,
	file_sets_writer: Pin<Box<BinaryCopyInWriter>>,
	connections_writer: Pin<Box<BinaryCopyInWriter>>,
	all_user_roles: FnvHashSet<(ulid::Ulid, ulid::Ulid)>,
	all_connections: FnvHashSet<(Platform, String)>,
}

impl Job for UsersJob {
	type T = types::User;

	const NAME: &'static str = "transfer_users";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating users, user_connections and user_roles tables");
			scuffle_utils::database::query("TRUNCATE users, user_connections, user_roles")
				.build()
				.execute(global.db())
				.await?;

			tracing::info!("deleting profile picture files from file_sets table");
			scuffle_utils::database::query("DELETE FROM file_sets WHERE kind = 'PROFILE_PICTURE'")
				.build()
				.execute(global.db())
				.await?;
		}

		tracing::info!("querying all entitlements");
		let mut entitlements_cursor = global
			.mongo()
			.database("7tv")
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
		tracing::info!("queried all entitlements");

		let users_client = global.db().get().await?;
		let users_writer = BinaryCopyInWriter::new(users_client
			.copy_in("COPY users (id, email, active_badge_id, active_paint_id, pending_profile_picture_file_set_id, active_profile_picture_file_set_id) FROM STDIN WITH (FORMAT BINARY)")
			.await?, &[Type::UUID, Type::VARCHAR, Type::UUID, Type::UUID, Type::UUID, Type::UUID]);

		let user_roles_client = global.db().get().await?;
		let user_roles_writer = BinaryCopyInWriter::new(
			user_roles_client
				.copy_in("COPY user_roles (user_id, role_id, added_at) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::UUID, Type::TIMESTAMPTZ],
		);

		let file_sets_client = global.db().get().await?;
		let file_sets_writer = BinaryCopyInWriter::new(
			file_sets_client
				.copy_in("COPY file_sets (id, kind, authenticated, properties) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, file_set_kind_type(&global).await?, Type::BOOL, Type::JSONB],
		);

		let connections_client = global.db().get().await?;
		let connections_writer = BinaryCopyInWriter::new(connections_client
			.copy_in("COPY user_connections (id, user_id, main_connection, platform_kind, platform_id, platform_username, platform_display_name, platform_avatar_url) FROM STDIN WITH (FORMAT BINARY)")
			.await?, &[Type::UUID, Type::UUID, Type::BOOL, platform_enum_type(&global).await?, Type::VARCHAR, Type::VARCHAR, Type::VARCHAR, Type::VARCHAR]);

		tracing::info!("created writers");

		Ok(Self {
			global,
			entitlements,
			users_writer: Box::pin(users_writer),
			user_roles_writer: Box::pin(user_roles_writer),
			file_sets_writer: Box::pin(file_sets_writer),
			connections_writer: Box::pin(connections_writer),
			all_user_roles: FnvHashSet::default(),
			all_connections: FnvHashSet::default(),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("users")
	}

	async fn process(&mut self, user: Self::T) -> ProcessOutcome {
		let user_id = user.id.into_ulid();

		let entitlements = self.entitlements.remove(&user.id).unwrap_or_default();

		for (id, ref_id) in entitlements.iter().filter_map(|e| match &e.data {
			types::EntitlementData::Role { ref_id } => Some((e.id, ref_id)),
			_ => None,
		}) {
			let role_id = ref_id.into_ulid();
			if self.all_user_roles.insert((user_id, role_id)) {
				if let Err(e) = self
					.user_roles_writer
					.as_mut()
					.write(&[&user_id, &role_id, &chrono::DateTime::from_timestamp(id.timestamp() as i64, 0)])
					.await
				{
					return ProcessOutcome {
						errors: vec![e.into()],
						inserted_rows: 0,
					};
				}
			}
		}

		let active_badge_id = entitlements
			.iter()
			.find(|e| matches!(e.data, types::EntitlementData::Badge { selected: true, .. }))
			.map(|e| e.id.into_ulid());

		let active_paint_id = entitlements
			.iter()
			.find(|e| matches!(e.data, types::EntitlementData::Paint { selected: true, .. }))
			.map(|e| e.id.into_ulid());

		let (pending_profile_picture_file_set_id, active_profile_picture_file_set_id) = match user.avatar {
			Some(types::UserAvatar::Processed {
				id,
				input_file,
				image_files,
			}) => {
				let file_set_id = id.into_ulid();

				let outputs = match image_files_to_file_properties(image_files) {
					Ok(outputs) => outputs,
					Err(e) => {
						return ProcessOutcome {
							errors: vec![e.into()],
							inserted_rows: 0,
						};
					}
				};

				if let Err(e) = self
					.file_sets_writer
					.as_mut()
					.write(&[
						&ulid::Ulid::from(file_set_id),
						&shared::database::FileSetKind::ProfilePicture,
						&false,
						&postgres_types::Json(shared::database::FileSetProperties::Image {
							input: input_file.into(),
							pending: false,
							outputs,
						}),
					])
					.await
				{
					return ProcessOutcome {
						errors: vec![e.into()],
						inserted_rows: 0,
					};
				}

				(None, Some(file_set_id))
			}
			Some(types::UserAvatar::Pending { pending_id }) => (Some(pending_id.into_ulid()), None),
			_ => (None, None),
		};

		if let Err(e) = self
			.users_writer
			.as_mut()
			.write(&[
				&user_id,
				&user.email,
				&active_badge_id,
				&active_paint_id,
				&pending_profile_picture_file_set_id,
				&active_profile_picture_file_set_id,
			])
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
			let id = ulid::Ulid::from_datetime(connection.linked_at.into_chrono().into());

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
				match self
					.connections_writer
					.as_mut()
					.write(&[
						&id,
						&user_id,
						&(i == 0),
						&platform,
						&platform_id,
						&platform_username,
						&platform_display_name,
						&platform_avatar_url,
					])
					.await
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

	async fn finish(mut self) -> anyhow::Result<()> {
		tracing::info!("finishing users job");

		self.users_writer.as_mut().finish().await?;
		tracing::info!("finished writing users");

		self.user_roles_writer.as_mut().finish().await?;
		tracing::info!("finished writing user roles");

		self.file_sets_writer.as_mut().finish().await?;
		tracing::info!("finished writing profile picture file sets");

		self.connections_writer.as_mut().finish().await?;
		tracing::info!("finished writing user connections");

		Ok(())
	}
}
