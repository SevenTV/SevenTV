use std::sync::Arc;

use crate::global::Global;

pub async fn run(global: Arc<Global>) -> anyhow::Result<()> {
	shared::health::run(
		&global.ctx().clone(),
		&global.config().health.http.clone(),
		move |path: &str| {
			let status = matches!(global.nats().connection_state(), async_nats::connection::State::Connected)
				&& match path {
					"/capacity" => {
						if let Some(limit) = global.config().api.connection_target.or(global.config().api.connection_limit) {
							global.active_connections() < limit
						} else {
							true
						}
					}
					_ => true,
				};

			async move { Ok(status) }
		},
	)
	.await?;

	Ok(())
}
