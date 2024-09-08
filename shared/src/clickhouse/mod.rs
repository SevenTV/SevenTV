use crate::config::ClickhouseConfig;

pub mod emote_stat;

pub trait ClickhouseCollection: clickhouse::Row + Send + Sync {
	const COLLECTION_NAME: &'static str;
}

pub async fn init_clickhouse(client: &ClickhouseConfig) -> anyhow::Result<clickhouse::Client> {
	let client = clickhouse::Client::default()
		.with_url(client.uri.clone())
		.with_user(client.username.clone())
		.with_password(client.password.clone())
		.with_database(client.database.clone());

	Ok(client)
}
