use async_graphql::SDLExportOptions;
use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::settings::cli::Matches;
use tokio::fs;
use tokio::signal::unix::SignalKind;

use crate::config::Config;

mod config;
mod connections;
mod dataloader;
mod global;
mod http;
mod image_processor;
mod jwt;
mod search;
mod stripe_client;
mod sub_refresh_job;
mod transactions;

#[bootstrap]
async fn main(settings: Matches<Config>) {
	rustls::crypto::aws_lc_rs::default_provider().install_default().ok();

	if let Some(export_path) = settings.settings.export_schema_path {
		fs::write(
			&export_path,
			http::v3::gql::schema(None).sdl_with_options(
				SDLExportOptions::default()
					.federation()
					.include_specified_by()
					.sorted_arguments()
					.sorted_enum_items()
					.sorted_fields(),
			),
		)
		.await
		.expect("failed to write schema path");

		tracing::info!(path = ?export_path, "saved gql schema");

		return;
	}

	tracing::info!("starting api");

	let global = global::Global::new(settings.settings)
		.await
		.expect("failed to initialize global");

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let http_handle = tokio::spawn(http::run(global.clone()));
	let image_processor_handle = tokio::spawn(image_processor::run(global.clone()));

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
		r = image_processor_handle => tracing::warn!("image processor handler exited: {:?}", r),
		_ = shutdown => tracing::warn!("failed to cancel context in time, force exit"),
	}

	tracing::info!("stopping api");
	std::process::exit(0);
}
