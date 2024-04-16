use std::sync::Arc;

use crate::global::Global;

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	shared::health::run(&global.ctx().clone(), &global.config().health.http.clone(), move |_: &str| {
		let status = matches!(global.nats().connection_state(), async_nats::connection::State::Connected);
		async move { Ok(status) }
	})
	.await?;

	Ok(())
}
