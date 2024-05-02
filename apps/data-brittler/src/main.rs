use std::sync::Arc;

use scuffle_foundations::bootstrap::{bootstrap, Bootstrap};
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

mod config;
mod error;
mod format;
mod global;
mod jobs;
mod report;
mod types;

struct BootstrapWrapper(config::Config);

impl From<config::Config> for BootstrapWrapper {
	fn from(config: config::Config) -> Self {
		Self(config)
	}
}

impl Bootstrap for BootstrapWrapper {
	type Settings = config::Config;

	fn telemetry_config(&self) -> Option<scuffle_foundations::telemetry::settings::TelemetrySettings> {
		Some(self.0.telemetry.clone())
	}

	fn runtime_mode(&self) -> scuffle_foundations::bootstrap::RuntimeSettings {
		self.0.runtime.clone()
	}
}

#[bootstrap]
async fn main(settings: Matches<BootstrapWrapper>) {
	tracing::info!("starting data-brittler");

	let global = Arc::new(
		global::Global::new(settings.settings.0)
			.await
			.expect("failed to initialize global"),
	);

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let jobs_handle = tokio::spawn(jobs::run(global.clone()));

	tokio::select! {
		_ = signal.recv() => {},
		r = jobs_handle => match r {
			Err(e) => tracing::error!("failed to spawn jobs: {e:?}"),
			Ok(Err(e)) => tracing::error!("failed to run jobs: {e:?}"),
			_ => {},
		},
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

	tracing::info!("stopping data-brittler");
	std::process::exit(0);
}
