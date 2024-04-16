use std::sync::Arc;

use cap::Cap;
use scuffle_utils::context::Context;
use scuffle_utils::prelude::FutureTimeout;
use tokio::signal::unix::SignalKind;

mod config;
mod global;
mod health;
mod http;
mod metrics;
mod subscription;
mod utils;

/// We use jemalloc as our global allocator, because the default system
/// allocator fragments memory too much. We noticed this that after some time
/// the resident memory usage would increase, but the allocated memory would
/// not. This is because the system allocator would allocate memory in small
/// chunks, but never free them. This is not a problem with jemalloc, because it
/// will free memory when it is no longer used. You can configure Jemalloc by
/// setting the MALLOC_CONF or _RJEM_MALLOC_CONF environment variables.
/// We also have a Cap on the allocator, this allows us to set a hard limit on
/// the amount of memory that can be allocated. Also allows us to get metrics
/// about the amount of memory that is allocated.
#[global_allocator]
static ALLOCATOR: Cap<tikv_jemallocator::Jemalloc> = Cap::new(tikv_jemallocator::Jemalloc, usize::max_value());

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

	tracing::info!("starting event-api");

	let (ctx, handler) = Context::new();

	let global = Arc::new(global::Global::new(ctx, config).await.expect("failed to initialize global"));

	let mut signal = scuffle_utils::signal::SignalHandler::new()
		.with_signal(SignalKind::interrupt())
		.with_signal(SignalKind::terminate());

	let app_handle = tokio::spawn(http::run(global.clone()));
	let health_handle = tokio::spawn(health::run(global.clone()));
	let metrics_handle = tokio::spawn(metrics::run(global.clone()));
	let subscription_handle = tokio::spawn(subscription::run(global.clone()));

	tokio::select! {
		_ = signal.recv() => tracing::info!("received shutdown signal"),
		r = subscription_handle => tracing::warn!("subscription manager exited: {:?}", r),
		r = app_handle => tracing::warn!("http server exited: {:?}", r),
		r = health_handle => tracing::warn!("health server exited: {:?}", r),
		r = metrics_handle => tracing::warn!("metrics server exited: {:?}", r),
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

	tracing::info!("stopping event-api");
	std::process::exit(0);
}
