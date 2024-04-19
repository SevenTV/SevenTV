use std::sync::Arc;

use cap::Cap;
use scuffle_utils::context::Context;
use scuffle_utils::prelude::FutureTimeout;
use tokio::signal::unix::SignalKind;

mod config;
mod error;
mod format;
mod global;
mod jobs;
mod report;
mod types;

#[global_allocator]
static ALLOCATOR: Cap<tikv_jemallocator::Jemalloc> = Cap::new(tikv_jemallocator::Jemalloc, usize::MAX);

#[tokio::main]
async fn main() {
	let config = shared::config::parse(true, Some("config".into())).expect("failed to parse config");
	shared::logging::init(&config.logging.level, config.logging.mode).expect("failed to initialize logging");

	if let Some(path) = config.config_file.as_ref() {
		tracing::info!("using config file: {path}");
	}

	if let Some(limit) = config.memory.limit {
		tracing::info!("setting memory limit to {limit} bytes");
		ALLOCATOR.set_limit(limit).expect("failed to set memory limit");
	}

	tracing::info!("starting data-brittler");

	let (ctx, handler) = Context::new();

	let global = Arc::new(global::Global::new(ctx, config).await.expect("failed to initialize global"));

	let mut signal = scuffle_utils::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let memuse = async {
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(30)).await;
			tracing::info!("memory usage: {} MiB", ALLOCATOR.allocated() / 1024 / 1204);
		}
	};

	let jobs_handle = tokio::spawn(jobs::run(global.clone()));

	tokio::select! {
		_ = signal.recv() => {},
		_ = memuse => {},
		r = jobs_handle => match r {
			Err(e) => tracing::error!("failed to spawn jobs: {e:?}"),
			Ok(Err(e)) => tracing::error!("failed to run jobs: {e:?}"),
			_ => {},
		},
	}

	drop(global);

	tokio::select! {
		_ = signal.recv() => tracing::info!("received second shutdown signal, forcing exit"),
		r = handler.cancel().timeout(std::time::Duration::from_secs(60)) => {
			if r.is_err() {
				tracing::warn!("failed to cancel context in time, force exit");
			}
		}
	}

	tracing::info!("stopping data-brittler");
	std::process::exit(0);
}
