pub mod graph;
mod id;
mod types;

pub use id::*;
pub use types::*;

use crate::config::DatabaseConfig;

pub async fn setup_database(config: &DatabaseConfig) -> anyhow::Result<mongodb::Client> {
	let options = mongodb::options::ClientOptions::parse(&config.uri).await?;

	let client = mongodb::Client::with_options(options)?;

	let db = client
		.default_database()
		.ok_or_else(|| anyhow::anyhow!("No default database"))?;

	init_database(&db).await?;

	Ok(client)
}
