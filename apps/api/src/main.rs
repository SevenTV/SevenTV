use async_graphql::SDLExportOptions;
use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::settings::cli::Matches;
use tokio::fs;
use tokio::signal::unix::SignalKind;

use crate::config::Config;

mod config;
mod connections;
mod cron;
mod dataloader;
mod global;
mod http;
mod image_processor;
mod jwt;
mod ratelimit;
mod search;
mod stripe_client;
mod sub_refresh_job;
mod transactions;
mod paypal_api;

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

	tracing::info!("starting api with {:?}", settings.settings.runtime);

	scuffle_foundations::telemetry::server::require_health_check();

	let global = global::Global::new(settings.settings)
		.await
		.expect("failed to initialize global");

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let http_handle = tokio::spawn(http::run(global.clone()));
	let image_processor_handle = tokio::spawn(image_processor::run(global.clone()));
	let cron_handle = tokio::spawn(cron::run(global.clone()));

	let handler = scuffle_foundations::context::Handler::global();

	let shutdown = tokio::spawn(async move {
		signal.recv().await;
		tracing::info!("received shutdown signal, waiting for jobs to finish");
		tokio::select! {
			_ = handler.shutdown() => {},
			_ = signal.recv() => {
				tracing::warn!("received second shutdown signal, forcing exit");
			},
			_ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
				tracing::warn!("shutdown timed out, forcing exit");
			},
		}
	});

	tracing::info!("started api");

	let ctx = scuffle_foundations::context::Context::global();

	tokio::select! {
		r = http_handle => {
			if !ctx.is_done() {
				tracing::warn!("http server exited: {:?}", r);
			}
		},
		r = image_processor_handle => {
			if !ctx.is_done() {
				tracing::warn!("image processor handler exited: {:?}", r);
			}
		},
		r = cron_handle => {
			if !ctx.is_done() {
				tracing::warn!("cron handler exited: {:?}", r);
			}
		},
	}

	drop(ctx);

	shutdown.await.expect("shutdown failed");

	tracing::info!("stopping api");
	std::process::exit(0);
}
