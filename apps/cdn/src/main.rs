use std::sync::Arc;

use anyhow::Context;
use config::Config;
use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

mod cache;
mod cdn_purge;
mod config;
mod global;
mod http;
mod metrics;

#[bootstrap]
async fn main(settings: Matches<Config>) -> anyhow::Result<()> {
	rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

	tracing::info!("starting cdn");

	let global = Arc::new(
		global::Global::new(settings.settings)
			.await
			.context("failed to create global")?,
	);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let app_handle = scuffle_foundations::runtime::spawn(http::run(global.clone()));
	let purge_handle = scuffle_foundations::runtime::spawn(cdn_purge::run(global.clone()));

	let metrics_handle = if global.config.telemetry.metrics.enabled {
		Some(scuffle_foundations::runtime::spawn(metrics::recorder()))
	} else {
		None
	};

	let handler = scuffle_foundations::context::Handler::global();

	let mut shutdown = tokio::spawn(async move {
		tokio::select! {
			_ = signal.recv() => {},
			_ = handler.done() => {},
		}

		tracing::info!("received shutdown signal, waiting for jobs to finish");
		tokio::select! {
			_ = handler.shutdown() => {
				tracing::info!("shutdown complete");
			},
			_ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
				tracing::warn!("timeout while waiting for jobs to finish, forcing exit");
				std::process::exit(1);
			},
			_ = signal.recv() => {
				tracing::warn!("received second shutdown signal, forcing exit");
				std::process::exit(1);
			},
		}
	});

	tokio::select! {
		r = app_handle => {
			match r {
				Ok(Err(err)) => {
					tracing::error!("http server exited: {:#}", err);
				}
				Err(err) => {
					tracing::error!("http server exited: {:#}", err);
				}
				Ok(Ok(())) => {
					tracing::info!("http server exited");
				}
			}

			handler.cancel();
		},
		Some(r) = async {
			if let Some(handle) = metrics_handle {
				Some(handle.await)
			} else {
				None
			}
		} => {
			match r {
				Ok(()) => {
					tracing::info!("metrics recorder exited");
				}
				Err(err) => {
					tracing::error!("metrics recorder exited: {:#}", err);
				}
			}
		},
		r = purge_handle => {
			match r {
				Ok(Err(err)) => {
					tracing::error!("purge worker exited: {:#}", err);
				}
				Err(err) => {
					tracing::error!("purge worker exited: {:#}", err);
				}
				Ok(Ok(())) => {
					tracing::info!("purge worker exited");
				}
			}
		},
		s = &mut shutdown => {
			if let Err(err) = s {
				tracing::error!("shutdown error: {:#}", err);
				std::process::exit(1);
			}

			std::process::exit(0);
		}
	}

	handler.cancel();
	shutdown.await.unwrap();

	Ok(())
}
