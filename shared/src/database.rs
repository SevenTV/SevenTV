use std::io;
use std::sync::Arc;

use anyhow::Context;
use rustls::RootCertStore;
use scuffle_utils::database::deadpool_postgres::{ManagerConfig, PoolConfig, RecyclingMethod, Runtime};
use scuffle_utils::database::tokio_postgres::NoTls;
use scuffle_utils::database::Pool;

use crate::config::DatabaseConfig;

pub async fn setup_database(config: &DatabaseConfig) -> anyhow::Result<Arc<Pool>> {
	let mut pg_config = config
		.uri
		.parse::<scuffle_utils::database::tokio_postgres::Config>()
		.context("invalid database uri")?;

	pg_config.ssl_mode(if config.tls.is_some() {
		scuffle_utils::database::tokio_postgres::config::SslMode::Require
	} else {
		scuffle_utils::database::tokio_postgres::config::SslMode::Disable
	});

	let manager = if let Some(tls) = &config.tls {
		let cert = tokio::fs::read(&tls.cert).await.context("failed to read redis client cert")?;
		let key = tokio::fs::read(&tls.key)
			.await
			.context("failed to read redis client private key")?;

		let key = rustls_pemfile::pkcs8_private_keys(&mut io::BufReader::new(io::Cursor::new(key)))
			.next()
			.ok_or_else(|| anyhow::anyhow!("failed to find private key in redis client private key file"))??
			.into();

		let certs = rustls_pemfile::certs(&mut io::BufReader::new(io::Cursor::new(cert))).collect::<Result<Vec<_>, _>>()?;

		let mut cert_store = RootCertStore::empty();
		if let Some(ca_cert) = &tls.ca_cert {
			let ca_cert = tokio::fs::read(ca_cert).await.context("failed to read redis ca cert")?;
			let ca_certs =
				rustls_pemfile::certs(&mut io::BufReader::new(io::Cursor::new(ca_cert))).collect::<Result<Vec<_>, _>>()?;
			for cert in ca_certs {
				cert_store.add(cert).context("failed to add redis ca cert")?;
			}
		}

		let tls = rustls::ClientConfig::builder()
			.with_root_certificates(cert_store)
			.with_client_auth_cert(certs, key)
			.context("failed to create redis tls config")?;

		scuffle_utils::database::deadpool_postgres::Manager::from_config(
			pg_config,
			tokio_postgres_rustls::MakeRustlsConnect::new(tls),
			ManagerConfig {
				recycling_method: RecyclingMethod::Fast,
			},
		)
	} else {
		scuffle_utils::database::deadpool_postgres::Manager::from_config(
			pg_config,
			NoTls,
			ManagerConfig {
				recycling_method: RecyclingMethod::Fast,
			},
		)
	};

	Ok(Arc::new(
		Pool::builder(manager)
			.config(PoolConfig::default())
			.runtime(Runtime::Tokio1)
			.build()
			.context("failed to create database pool")?,
	))
}
