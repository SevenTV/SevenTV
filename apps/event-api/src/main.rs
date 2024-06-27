use std::sync::Arc;

use config::Config;
use scuffle_foundations::bootstrap::{bootstrap, Bootstrap, RuntimeSettings};
use scuffle_foundations::settings::cli::Matches;
use scuffle_foundations::telemetry::settings::TelemetrySettings;
use tokio::signal::unix::SignalKind;

mod config;
mod global;
mod http;
// mod metrics;
mod subscription;
mod utils;

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
	tracing::info!("starting event-api");

	let global = Arc::new(
		global::Global::new(settings.settings.0)
			.await
			.expect("failed to initialize global"),
	);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let app_handle = tokio::spawn(http::run(global.clone()));
	// let metrics_handle = tokio::spawn(metrics::run(global.clone()));
	let subscription_handle = tokio::spawn(subscription::run(global.clone()));

	let handler = scuffle_foundations::context::Handler::global();

	let shutdown = tokio::spawn(async move {
		signal.recv().await;
		tracing::info!("received shutdown signal, waiting for jobs to finish");
		handler.shutdown().await;
		tokio::time::timeout(std::time::Duration::from_secs(60), signal.recv())
			.await
			.ok();
	});

	tokio::select! {
		r = subscription_handle => tracing::warn!("subscription manager exited: {:?}", r),
		r = app_handle => tracing::warn!("http server exited: {:?}", r),
		// r = metrics_handle => tracing::warn!("metrics server exited: {:?}", r),
		_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
	}

	tracing::info!("stopping event-api");
	std::process::exit(0);
}
