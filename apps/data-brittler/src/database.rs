use std::sync::Arc;

use postgres_types::Type;

use crate::global::Global;

async fn any_enum_type(global: &Arc<Global>, name: &str, variants: Vec<String>) -> anyhow::Result<Type> {
	let oid: u32 = scuffle_utils::database::query("SELECT oid FROM pg_type WHERE typname = $1")
		.bind(name)
		.build()
		.fetch_one(global.db())
		.await?
		.get(0);
	Ok(Type::new(
		name.to_string(),
		oid,
		postgres_types::Kind::Enum(variants),
		"".to_string(),
	))
}

pub async fn platform_enum_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(
		global,
		"platform",
		vec![
			"DISCORD".to_string(),
			"TWITCH".to_string(),
			"GOOGLE".to_string(),
			"KICK".to_string(),
		],
	)
	.await
}

pub async fn file_set_kind_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(
		global,
		"file_set_kind",
		vec![
			"TICKET".to_string(),
			"PROFILE_PICTURE".to_string(),
			"BADGE".to_string(),
			"PAINT".to_string(),
			"EMOTE".to_string(),
			"PRODUCT".to_string(),
			"PAGE".to_string(),
		],
	)
	.await
}

pub async fn emote_set_kind_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(global, "emote_set_kind", vec!["NORMAL".to_string(), "PERSONAL".to_string()]).await
}

pub async fn ticket_priority_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(
		global,
		"ticket_priority",
		vec![
			"LOW".to_string(),
			"MEDIUM".to_string(),
			"HIGH".to_string(),
			"URGENT".to_string(),
		],
	)
	.await
}

pub async fn ticket_kind_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(
		global,
		"ticket_kind",
		vec![
			"EMOTE_REPORT".to_string(),
			"USER_REPORT".to_string(),
			"BILLING".to_string(),
			"EMOTE_LISTING_REQUEST".to_string(),
			"EMOTE_PERSONAL_USE_REQUEST".to_string(),
			"OTHER".to_string(),
		],
	)
	.await
}

pub async fn ticket_member_kind_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(
		global,
		"ticket_member_kind",
		vec!["OP".to_string(), "MEMBER".to_string(), "STAFF".to_string()],
	)
	.await
}

pub async fn ticket_status_type(global: &Arc<Global>) -> anyhow::Result<Type> {
	any_enum_type(
		global,
		"ticket_status",
		vec![
			"PENDING".to_string(),
			"IN_PROGRESS".to_string(),
			"FIXED".to_string(),
			"CLOSED".to_string(),
		],
	)
	.await
}
