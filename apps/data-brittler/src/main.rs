use std::sync::Arc;

use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

use self::config::Config;

mod config;
mod download_cosmetics;
mod error;
mod format;
mod global;
mod image_processor_callback;
mod jobs;
mod report;
mod types;

#[bootstrap]
async fn main(settings: Matches<Config>) {
	tracing::info!("starting data-brittler");

	let global = Arc::new(
		global::Global::new(settings.settings)
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
