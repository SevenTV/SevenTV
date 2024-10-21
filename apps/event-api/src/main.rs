use std::sync::Arc;

use config::Config;
use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

mod config;
mod global;
mod http;
mod subscription;
mod utils;

#[bootstrap]
async fn main(settings: Matches<Config>) {
	rustls::crypto::aws_lc_rs::default_provider().install_default().ok();

	tracing::info!("starting event-api");

	let global = Arc::new(
		global::Global::new(settings.settings)
			.await
			.expect("failed to initialize global"),
	);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let app_handle = tokio::spawn(http::run(global.clone()));
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
		_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
	}

	tracing::info!("stopping event-api");
	std::process::exit(0);
}
