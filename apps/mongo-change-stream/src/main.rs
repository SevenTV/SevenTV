use std::sync::Arc;

use scuffle_foundations::bootstrap::bootstrap;
use scuffle_foundations::context::Context;
use scuffle_foundations::settings::cli::Matches;
use tokio::signal::unix::SignalKind;

use self::config::Config;

mod config;
mod global;
mod stream;

#[bootstrap]
async fn main(settings: Matches<Config>) -> anyhow::Result<()> {
	tracing::info!("starting mongo change stream");

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

	let mut shutdown = tokio::spawn(async move {
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

	let mut last_failure: Option<std::time::Instant> = None;
	let mut failure_count = 0;

	loop {
		let stream = tokio::spawn(stream::start(global.clone()));

		tokio::select! {
			r = &mut shutdown => {
				if let Err(e) = r {
					tracing::error!("shutdown error: {:#}", e);
				} else {
					tracing::info!("shutdown complete");
				}
			}
			r = stream => {
				match r {
					Ok(Ok(())) if !Context::global().is_done() => {
						tracing::error!("mongo stream stopped, without error, attempting to restart");

						if let Some(ts) = last_failure {
							if ts.elapsed() < std::time::Duration::from_secs(30) && failure_count > 5 {
								tracing::warn!("mongo stream stopped, without error, but within 30 seconds of last failure, delaying restart");
								tokio::time::sleep(std::time::Duration::from_secs(5)).await;
							} else {
								failure_count = 0;
							}
						}

						last_failure = Some(std::time::Instant::now());
						failure_count += 1;

						continue;
					}
					Ok(Err(e)) => {
						tracing::error!("mongo stream error: {:#}", e);
					}
					Err(e) => {
						tracing::error!("mongo stream error: {:#}", e);
					}
					Ok(Ok(())) => {}
				}
			}
		}

		break;
	}

	tracing::info!("stopping mongo change stream");

	std::process::exit(0);
}
