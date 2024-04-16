use std::pin::Pin;
use std::sync::Arc;

use postgres_types::Type;
use shared::database::RoleData;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use super::{Job, ProcessOutcome};
use crate::global::Global;
use crate::types;

pub struct RolesJob {
	global: Arc<Global>,
	roles_writer: Pin<Box<BinaryCopyInWriter>>,
}

impl Job for RolesJob {
	type T = types::Role;

	const NAME: &'static str = "transfer_roles";

	async fn new(global: Arc<Global>) -> anyhow::Result<Self> {
		if global.config().truncate {
			tracing::info!("truncating roles table");
			scuffle_utils::database::query("TRUNCATE roles")
				.build()
				.execute(global.db())
				.await?;
		}

		let roles_client = global.db().get().await?;
		let roles_writer = BinaryCopyInWriter::new(
			roles_client
				.copy_in("COPY roles (id, name, data, priority, color) FROM STDIN WITH (FORMAT BINARY)")
				.await?,
			&[Type::UUID, Type::VARCHAR, Type::JSONB, Type::INT2, Type::INT4],
		);

		Ok(RolesJob {
			global,
			roles_writer: Box::pin(roles_writer),
		})
	}

	async fn collection(&self) -> mongodb::Collection<Self::T> {
		self.global.mongo().database("7tv").collection("roles")
	}

	async fn process(&mut self, role: Self::T) -> super::ProcessOutcome {
		let mut outcome = ProcessOutcome::default();

		let id = role.id.into_ulid();

		let data = RoleData {
			permissions: role.to_new_permissions(),
			discord_id: role.discord_id,
		};

		let priority = role.position.try_into().unwrap_or(i16::MAX);

		match self
			.roles_writer
			.as_mut()
			.write(&[&id, &role.name, &postgres_types::Json(data), &priority, &role.color])
			.await
		{
			Ok(_) => {
				outcome.inserted_rows += 1;
			}
			Err(e) => {
				outcome.errors.push(e.into());
			}
		}

		outcome
	}

	async fn finish(mut self) -> anyhow::Result<()> {
		self.roles_writer.as_mut().finish().await?;
		Ok(())
	}
}
