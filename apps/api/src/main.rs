use std::sync::Arc;

use scuffle_foundations::bootstrap::{bootstrap, Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::cli::Matches;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use tokio::signal::unix::SignalKind;

use crate::config::Config;

mod config;
mod connections;
mod dataloader;
mod global;
mod http;
mod image_processor;
mod jwt;

struct BootstrapWrapper(Config);

impl From<Config> for BootstrapWrapper {
	fn from(config: Config) -> Self {
		Self(config)
	}
}

impl Bootstrap for BootstrapWrapper {
	type Settings = Config;

	fn telemetry_config(&self) -> Option<TelemetrySettings> {
		Some(self.0.telemetry.clone())
	}

	fn runtime_mode(&self) -> RuntimeSettings {
		self.0.runtime.clone()
	}
}

#[bootstrap]
async fn main(settings: Matches<BootstrapWrapper>) {
	tracing::info!("starting api");

	let global = Arc::new(
		global::Global::new(settings.settings.0)
			.await
			.expect("failed to initialize global"),
	);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let http_handle = tokio::spawn(http::run(global.clone()));

	tokio::select! {
		_ = signal.recv() => tracing::info!("received shutdown signal"),
		r = http_handle => tracing::warn!("http server exited: {:?}", r),
	}

	let handler = scuffle_foundations::context::Handler::global();

	tokio::select! {
		_ = signal.recv() => tracing::info!("received second shutdown signal, forcing exit"),
		r = tokio::time::timeout(std::time::Duration::from_secs(60), handler.shutdown()) => {
			if r.is_err() {
				tracing::warn!("failed to cancel context in time, force exit");
			}
		}
	}

	tracing::info!("stopping api");
	std::process::exit(0);
}
