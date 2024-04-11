use std::sync::Arc;

use postgres_types::Type;

use crate::global::Global;

pub async fn platform_enum_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	let oid: u32 = scuffle_utils::database::query("SELECT oid FROM pg_type WHERE typname = 'platform'")
		.build()
		.fetch_one(global.db())
		.await?
		.get(0);
	Ok(Type::new(
		"platform".to_string(),
		oid,
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
	let oid: u32 = scuffle_utils::database::query("SELECT oid FROM pg_type WHERE typname = 'file_set_kind'")
		.build()
		.fetch_one(global.db())
		.await?
		.get(0);
	Ok(Type::new(
		"file_set_kind".to_string(),
		oid,
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

pub async fn emote_set_kind_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	let oid: u32 = scuffle_utils::database::query("SELECT oid FROM pg_type WHERE typname = 'emote_set_kind'")
		.build()
		.fetch_one(global.db())
		.await?
		.get(0);
	Ok(Type::new(
		"emote_set_kind".to_string(),
		oid,
		postgres_types::Kind::Enum(vec!["NORMAL".to_string(), "PERSONAL".to_string()]),
		"".to_string(),
	))
}
