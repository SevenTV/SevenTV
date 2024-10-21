use futures::TryStreamExt;
use shared::{
	config::DatabaseConfig,
	database::{badge::Badge, queries::filter, MongoCollection},
};

#[tokio::main]
async fn main() {
	let config = DatabaseConfig { uri: "".to_string() };

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
}
