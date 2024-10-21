use scuffle_foundations::{
	bootstrap::{bootstrap, Bootstrap},
	settings::{auto_settings, cli::Matches},
};
use shared::config::DatabaseConfig;

mod badges;

#[auto_settings]
#[serde(default)]
struct Config {
	database: DatabaseConfig,
}

impl Bootstrap for Config {
	type Settings = Self;
}

#[bootstrap]
async fn main(settings: Matches<Config>) {
	let config = DatabaseConfig {
		uri: settings.settings.database.uri.clone(),
	};

	let mongo = shared::database::setup_database(&config, false).await.unwrap();
	let db = mongo.default_database().unwrap();

	for badge in badges::jobs() {
		println!("{:?}", badge);
	}

	std::process::exit(0);
}
