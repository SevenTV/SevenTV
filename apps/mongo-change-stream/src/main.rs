use std::sync::Arc;

use scuffle_foundations::{bootstrap::{bootstrap, Bootstrap}, context::Context, settings::{auto_settings, cli::Matches}};
use shared::config::{DatabaseConfig, NatsConfig};
use tokio::signal::unix::SignalKind;

mod global;
mod stream;

#[auto_settings]
struct Config {
	database: DatabaseConfig,
	nats: NatsConfig,
	telementry: scuffle_foundations::telemetry::settings::TelemetrySettings,
	stream_name: String,
	nats_prefix: String,
}

impl Bootstrap for Config {
	type Settings = Self;

	fn runtime_mode(&self) -> scuffle_foundations::bootstrap::RuntimeSettings {
		scuffle_foundations::bootstrap::RuntimeSettings::Steal {
			threads: 1,
			name: "mongo-change-stream".to_string(),
		}
	}

	fn telemetry_config(&self) -> Option<scuffle_foundations::telemetry::settings::TelemetrySettings> {
		Some(self.telementry.clone())
	}
}

#[bootstrap]
async fn main(settings: Matches<Config>) -> anyhow::Result<()> {
	tracing::info!("starting mongo change stream");

	let global = Arc::new(
		global::Global::new(settings.settings)
			.await
			.expect("failed to initialize global"),
	);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let handler = scuffle_foundations::context::Handler::global();

	let shutdown = tokio::spawn(async move {
		signal.recv().await;
		tracing::info!("received shutdown signal, waiting for jobs to finish");
		handler.cancel();
		if tokio::time::timeout(std::time::Duration::from_secs(60), async {
			tokio::select! {
				_ = handler.done() => {}
				_ = signal.recv() => {
					tracing::info!("received second shutdown signal, forcing exit");
				}
			}
		})
		.await
		.is_err()
		{
			tracing::info!("shutdown timeout, forcing exit");
		}
	});

	let triggers = tokio::spawn(stream::start(global.clone()));

	tokio::select! {
		r = shutdown => {
			if let Err(e) = r {
				tracing::error!("shutdown error: {:#}", e);
			} else {
				tracing::info!("shutdown complete");
			}
		}
		r = triggers => {
			match r {
				Ok(Ok(())) if !Context::global().is_done() => {
					tracing::warn!("mongo change stream stopped");
				}
				Ok(Err(e)) => {
					tracing::error!("mongo change stream error: {:#}", e);
				}
				Err(e) => {
					tracing::error!("mongo change stream error: {:#}", e);
				}
				Ok(Ok(())) => {}
			}
		}
	}

	tracing::info!("stopping mongo change stream");

	std::process::exit(0);
}
