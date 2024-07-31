use anyhow::Context;
use async_nats::ServerAddr;

use crate::config::NatsConfig;

pub async fn setup_nats(
	name: &str,
	config: &NatsConfig,
) -> anyhow::Result<(async_nats::Client, async_nats::jetstream::Context)> {
	let nats = {
		let mut options = async_nats::ConnectOptions::new().name(name).retry_on_initial_connect();

		if let Some(user) = &config.username {
			options = options.user_and_password(user.clone(), config.password.clone().unwrap_or_default())
		} else if let Some(token) = &config.token {
			options = options.token(token.clone())
		}

		if let Some(tls) = &config.tls {
			options = options
				.require_tls(true)
				.add_client_certificate((&tls.cert).into(), (&tls.key).into());

			if let Some(ca_cert) = &tls.ca_cert {
				options = options.add_root_certificates(ca_cert.into())
			}
		}

		options
			.connect(
				config
					.servers
					.iter()
					.map(|s| s.parse::<ServerAddr>())
					.collect::<Result<Vec<_>, _>>()
					.context("failed to parse nats server addresses")?,
			)
			.await
			.context("failed to connect to nats")?
	};

	let jetstream = async_nats::jetstream::new(nats.clone());

	Ok((nats, jetstream))
}

#[derive(Debug, Clone)]
pub struct ChangeStreamSubject(String);

fn escape_prefix(prefix: &str) -> String {
	prefix.trim_end_matches('-').replace('.', "_")
}

impl ChangeStreamSubject {
	pub fn new(prefix: &str) -> Self {
		Self(format!("{}::MongoChangeStream", escape_prefix(prefix)))
	}

	pub fn name(&self) -> String {
		self.0.clone()
	}

	pub fn wildcard(&self) -> String {
		format!("{}.>", self.0)
	}

	pub fn topic(&self, database: &str, collection: &str) -> String {
		format!("{}.{}.{}", self.0, database, collection)
	}
}
