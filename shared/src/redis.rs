use anyhow::Context;
use fred::prelude::{ClientLike, RedisPool};
use fred::types::{RedisConfig, Server, ServerConfig, TracingConfig};

pub async fn setup_redis(config: &crate::config::RedisConfig) -> anyhow::Result<RedisPool> {
	let server_config = match &config.sentinel_service_name {
		Some(sentinel_service_name) => ServerConfig::Sentinel {
			hosts: config
				.servers
				.iter()
				.map(|s| parse_server(s))
				.collect::<anyhow::Result<Vec<_>>>()?,
			password: config.password.clone(),
			username: config.username.clone(),
			service_name: sentinel_service_name.clone(),
		},
		None if config.servers.len() == 1 => ServerConfig::Centralized {
			server: parse_server(&config.servers[0])?,
		},
		None => ServerConfig::Clustered {
			hosts: config
				.servers
				.iter()
				.map(|s| parse_server(s))
				.collect::<anyhow::Result<Vec<_>>>()?,
			policy: Default::default(),
		},
	};

	let cfg = RedisConfig {
		server: server_config,
		database: Some(config.database),
		fail_fast: true,
		password: config.password.clone(),
		username: config.username.clone(),
		tracing: TracingConfig::new(true),
		..Default::default()
	};

	let client = RedisPool::new(cfg, None, None, None, config.max_connections).context("pool")?;

	client.init().await?;

	Ok(client)
}

fn parse_server(server: &str) -> anyhow::Result<Server> {
	let port_ip = server.split(':').collect::<Vec<_>>();

	if port_ip.len() == 1 {
		Ok(Server::new(port_ip[0], 6379))
	} else {
		Ok(Server::new(port_ip[0], port_ip[1].parse::<u16>().context("invalid port")?))
	}
}
