use crate::config::DatabaseConfig;

mod types;

pub use types::*;

pub async fn setup_database(config: &DatabaseConfig) -> anyhow::Result<mongodb::Client> {
	anyhow::bail!("Database setup not implemented");
}
