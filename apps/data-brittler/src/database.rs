use std::sync::Arc;

use postgres_types::Type;
use tokio_postgres::binary_copy::BinaryCopyInWriter;

use crate::global::Global;

pub async fn file_sets_writer(global: &Arc<Global>) -> anyhow::Result<BinaryCopyInWriter> {
	let file_sets_client = global.db().get().await?;
	Ok(BinaryCopyInWriter::new(
		file_sets_client
			.copy_in("COPY file_sets (id, kind, authenticated, properties) FROM STDIN WITH (FORMAT BINARY)")
			.await?,
		&[Type::UUID, file_set_kind_type(&global).await?, Type::BOOL, Type::JSONB],
	))
}

pub async fn platform_enum_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	let platform_type_oid: u32 = scuffle_utils::database::query("SELECT oid FROM pg_type WHERE typname = 'platform'")
		.build()
		.fetch_one(global.db())
		.await?
		.get(0);
	Ok(Type::new(
		"platform".to_string(),
		platform_type_oid,
		postgres_types::Kind::Enum(vec![
			"DISCORD".to_string(),
			"TWITCH".to_string(),
			"GOOGLE".to_string(),
			"KICK".to_string(),
		]),
		"".to_string(),
	))
}

pub async fn file_set_kind_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	let file_set_kind_type_oid: u32 =
		scuffle_utils::database::query("SELECT oid FROM pg_type WHERE typname = 'file_set_kind'")
			.build()
			.fetch_one(global.db())
			.await?
			.get(0);
	Ok(Type::new(
		"file_set_kind".to_string(),
		file_set_kind_type_oid,
		postgres_types::Kind::Enum(vec![
			"TICKET".to_string(),
			"PROFILE_PICTURE".to_string(),
			"BADGE".to_string(),
			"PAINT".to_string(),
			"EMOTE".to_string(),
			"PRODUCT".to_string(),
			"PAGE".to_string(),
		]),
		"".to_string(),
	))
}
