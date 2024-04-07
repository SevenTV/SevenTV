use std::pin::Pin;
use std::sync::Arc;

use fnv::FnvHashSet;
use postgres_from_row::tokio_postgres::binary_copy::BinaryCopyInWriter;
use postgres_types::Type;
use shared::database::Platform;

use super::{Job, ProcessOutcome};
use crate::database::{file_set_kind_type, platform_enum_type};
use crate::global::Global;
use crate::types::image_files_to_file_properties;
use crate::{error, types};

pub struct UsersJob {
	global: Arc<Global>,
	users_writer: Pin<Box<BinaryCopyInWriter>>,
	file_sets_writer: Pin<Box<BinaryCopyInWriter>>,
	connections_writer: Pin<Box<BinaryCopyInWriter>>,
	all_connections: FnvHashSet<(Platform, String)>,
}

impl UsersJob {
	pub async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating users and user_connections tables");
			scuffle_utils::database::query("TRUNCATE users, user_connections")
				.build()
				.execute(global.db())
				.await?;

			tracing::info!("deleting profile picture files from file_sets table");
			scuffle_utils::database::query("DELETE FROM file_sets WHERE kind = 'PROFILE_PICTURE'")
				.build()
				.execute(global.db())
				.await?;
		}

		let users_client = global.db().get().await?;
		// TODO: active paint, badge, profile picture
		let users_writer = BinaryCopyInWriter::new(users_client
			.copy_in("COPY users (id, email, pending_profile_picture_file_set_id, active_profile_picture_file_set_id) FROM STDIN WITH (FORMAT BINARY)")
			.await?, &[Type::UUID, Type::VARCHAR, Type::UUID, Type::UUID]);

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

		Ok(Self {
			global,
			users_writer: Box::pin(users_writer),
			file_sets_writer: Box::pin(file_sets_writer),
			connections_writer: Box::pin(connections_writer),
			all_connections: FnvHashSet::default(),
		})
	}
}

impl Job for UsersJob {
	type T = types::User;

	const NAME: &'static str = "transfer_users";

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("users")
	}

	async fn process(&mut self, user: Self::T) -> ProcessOutcome {
		let user_id = user.id.into_ulid();

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

		self.file_sets_writer.as_mut().finish().await?;
		tracing::info!("finished writing profile picture file sets");

		self.connections_writer.as_mut().finish().await?;
		tracing::info!("finished writing user connections");

		Ok(())
	}
}
