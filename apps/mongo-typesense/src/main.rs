use std::sync::Arc;

use rand::Rng;
use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::context::Context;
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

use self::config::Config;

mod batcher;
mod config;
mod global;
mod types;
mod typesense;

#[bootstrap]
async fn main(settings: Matches<Config>) -> anyhow::Result<()> {
	rustls::crypto::aws_lc_rs::default_provider().install_default().ok();

	tracing::info!("starting mongo typesense");

	let global = Arc::new(
		global::Global::new(settings.settings)
			.await
			.expect("failed to initialize global"),
	);

	scuffle_foundations::telemetry::server::register_health_check(global.clone());

	let mut signal = scuffle_foundations::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let handler = scuffle_foundations::context::Handler::global();

	let shutdown = tokio::spawn(async move {
		signal.recv().await;
		tracing::info!("received shutdown signal, waiting for jobs to finish");
		handler.cancel();
		if tokio::time::timeout(std::time::Duration::from_secs(60), async {
			tokio::select! {
				_ = handler.done() => {}
				_ = signal.recv() => {
					tracing::info!("received second shutdown signal, forcing exit");
				}
			}
		})
		.await
		.is_err()
		{
			tracing::info!("shutdown timeout, forcing exit");
		}
	});

	let triggers = tokio::spawn(typesense::start(global.clone()));

	tokio::spawn({
		let global = global.clone();
		async move {
			let mut last_requests = global.request_count();

			fn jitter() -> std::time::Duration {
				let mut rng = rand::thread_rng();
				let secs = rng.gen_range(0..10);
				std::time::Duration::from_secs(secs)
			}

			loop {
				tokio::time::sleep(std::time::Duration::from_secs(60) + jitter()).await;
				if !global.wait_healthy().await {
					continue;
				}

				let new_requests = global.request_count();
				let delta = new_requests - last_requests;
				last_requests = new_requests;

				// We are likely not busy processing requests so we should attempt to check if
				// any objects are not correctly indexed
				if delta < 1000 {
					global.reindex().await;
				}
			}
		}
	});

	tokio::select! {
		r = shutdown => {
			if let Err(e) = r {
				tracing::error!("shutdown error: {:#}", e);
			} else {
				tracing::info!("shutdown complete");
			}
		}
		r = triggers => {
			match r {
				Ok(Ok(())) if !Context::global().is_done() => {
					tracing::warn!("mongo typesense stopped");
				}
				Ok(Err(e)) => {
					tracing::error!("mongo triggers error: {:#}", e);
				}
				Err(e) => {
					tracing::error!("mongo triggers error: {:#}", e);
				}
				Ok(Ok(())) => {}
			}
		}
	}

	tracing::info!("stopping mongo typesense");

	std::process::exit(0);
}
