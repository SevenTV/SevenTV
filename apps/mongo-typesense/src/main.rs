use std::sync::Arc;

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
			let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
			loop {
				interval.tick().await;
				global.log_stats().await;
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
