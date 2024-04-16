use std::sync::Arc;

use crate::config::DatabaseConfig;

mod types;

pub use types::*;

pub async fn setup_database(config: &DatabaseConfig) -> anyhow::Result<()> {
	anyhow::bail!("Database setup not implemented");
}
