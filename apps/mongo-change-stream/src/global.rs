use anyhow::Context;
use scuffle_foundations::telemetry::server::HealthCheck;

pub struct Global {
	pub db: mongodb::Database,
    pub nats: async_nats::Client,
    pub config: crate::Config,
    pub jetstream: async_nats::jetstream::Context,
}

impl Global {
	pub async fn new(config: crate::Config) -> anyhow::Result<Self> {
        let (nats, jetstream) = shared::nats::setup_nats("mongo-change-stream", &config.nats).await.context("nats connect")?;

        let db = mongodb::Client::with_uri_str(&config.database.uri)
            .await
            .context("mongo connect")?
            .default_database()
            .ok_or_else(|| anyhow::anyhow!("no default database"))?;


        Ok(Self { db, nats, config, jetstream })
    }
}

impl HealthCheck for Global {
    fn check(&self) -> std::pin::Pin<Box<dyn futures::Future<Output = bool> + Send + '_>> {
        Box::pin(async { 
            matches!(self.nats.connection_state(), async_nats::connection::State::Connected) && self.db.run_command(bson::doc! { "ping": 1 }).await.is_ok()
         })
    }
}
