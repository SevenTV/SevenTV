use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

use crate::config::Config;

mod config;
mod connections;
mod dataloader;
mod event_api;
mod global;
mod http;
mod image_processor_callback;
mod jwt;
mod mongo_updater;
mod queries;
mod utils;

#[bootstrap]
async fn main(settings: Matches<Config>) {
	tracing::info!("starting api");

	let global = global::Global::new(settings.settings)
		.await
		.expect("failed to initialize global");

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let http_handle = tokio::spawn(http::run(global.clone()));
	let image_processor_callback_handle = tokio::spawn(image_processor_callback::run(global.clone()));

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
		r = http_handle => tracing::warn!("http server exited: {:?}", r),
		r = image_processor_callback_handle => tracing::warn!("image processor callback handler exited: {:?}", r),
		_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
	}

	tracing::info!("stopping api");
	std::process::exit(0);
}
