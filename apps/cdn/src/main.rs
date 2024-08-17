use std::sync::Arc;

use config::Config;
use scuffle_foundations::{bootstrap::bootstrap, settings::cli::Matches};
use tokio::signal::unix::SignalKind;

mod cache;
mod config;
mod global;
mod http;

#[bootstrap]
async fn main(settings: Matches<Config>) {
	tracing::info!("starting cdn");

	let global = Arc::new(global::Global::new(settings.settings).await);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let app_handle = tokio::spawn(http::run(global.clone()));

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
		r = app_handle => tracing::warn!("http server exited: {:?}", r),
		_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
	}

	tracing::info!("stopping cdn");
	std::process::exit(0);
}
