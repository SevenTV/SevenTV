use std::sync::Arc;

use global::Global;
use rand::Rng;
use scuffle_bootstrap::signals::SignalSvc;
use scuffle_context::ContextFutExt;

mod batcher;
mod config;
mod global;
mod types;
mod typesense;

scuffle_bootstrap::main! {
	Global {
		typesense::run,
		refresh,
		SignalSvc,
	}
}

async fn refresh(global: Arc<Global>, ctx: scuffle_context::Context) -> anyhow::Result<()> {
	let mut last_requests = global.request_count();

	fn jitter() -> std::time::Duration {
		let mut rng = rand::thread_rng();
		let secs = rng.gen_range(0..10);
		std::time::Duration::from_secs(secs)
	}

	async {
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
	.with_context(ctx)
	.await;

	Ok(())
}
