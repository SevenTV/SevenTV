use std::sync::Arc;

use scuffle_foundations::bootstrap::{bootstrap, Bootstrap};
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

mod config;
mod download_cosmetics;
mod error;
mod format;
mod global;
mod image_processor_callback;
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

	let handler = scuffle_foundations::context::Handler::global();

	let shutdown = tokio::spawn(async move {
		signal.recv().await;
		tracing::info!("received shutdown signal, waiting for jobs to finish");
		handler.shutdown().await;
		tokio::time::timeout(std::time::Duration::from_secs(60), signal.recv())
			.await
			.ok();
	});

	if global.config().download_cosmetics {
		let job = download_cosmetics::run(global.clone());

		tokio::select! {
			r = job => match r {
				Ok(_) => tracing::info!("finished running cosmetics download job"),
				Err(e) => tracing::error!(error = %e, "failed to run job"),
			},
			_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
		}
	} else {
		let joined_jobs = futures::future::join(jobs::run(global.clone()), image_processor_callback::run(global.clone()));

		tokio::select! {
			r = joined_jobs => match r {
				(Err(e), _) => tracing::error!(error = %e, "failed to run jobs"),
				(_, Err(e)) => tracing::error!(error = %e, "failed to run image processor callback"),
				_ => {},
			},
			_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
		}
	}

	tracing::info!("stopping data-brittler");
	std::process::exit(0);
}
