use futures::TryStreamExt;
use scuffle_foundations::{
	bootstrap::{bootstrap, Bootstrap},
	settings::{auto_settings, cli::Matches},
};
use shared::{
	config::DatabaseConfig,
	database::{badge::Badge, queries::filter, MongoCollection},
};

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

	let mut badges = Badge::collection(&db)
		.find(filter::filter! {
			Badge {}
		})
		.await
		.unwrap();

	while let Some(badge) = badges.try_next().await.unwrap() {
		println!("{:?}", badge);
	}

	std::process::exit(0);
}
