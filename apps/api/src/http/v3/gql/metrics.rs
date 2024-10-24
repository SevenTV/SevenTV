use std::sync::Arc;

use async_graphql::extensions::{Extension, ExtensionFactory};
use async_graphql::PathSegment;
use scuffle_foundations::telemetry::metrics::metrics;

pub struct ErrorMetrics;

impl ExtensionFactory for ErrorMetrics {
	fn create(&self) -> Arc<dyn async_graphql::extensions::Extension> {
		Arc::new(ErrorMetricsExtension)
	}
}

struct ErrorMetricsExtension;

#[metrics]
mod gql_v3 {
	use scuffle_foundations::telemetry::metrics::prometheus_client::metrics::counter::Counter;

	pub fn request(path: String) -> Counter;
	pub fn error(path: String, code: String, status_code: String) -> Counter;
}

fn path_segment_display(segments: &[PathSegment]) -> String {
	let mut path = String::new();

	for segment in segments {
		match segment {
			PathSegment::Field(field) => {
				if !path.is_empty() {
					path.push('.');
				}

				path.push_str(field);
			}
			PathSegment::Index(index) => {
				path.push('[');
				path.push_str(&index.to_string());
				path.push(']');
			}
		}
	}

	path
}

fn handle_error(error: &async_graphql::ServerError) {
	let code = error
		.extensions
		.as_ref()
		.and_then(|ext| ext.get("code"))
		.map(|c| c.to_string());
	let status_code = error
		.extensions
		.as_ref()
		.and_then(|ext| ext.get("status"))
		.map(|c| c.to_string());

	gql_v3::error(
		path_segment_display(&error.path),
		code.unwrap_or_default(),
		status_code.unwrap_or_default(),
	)
	.inc();
}

#[async_trait::async_trait]
impl Extension for ErrorMetricsExtension {
	async fn request(
		&self,
		ctx: &async_graphql::extensions::ExtensionContext<'_>,
		next: async_graphql::extensions::NextRequest<'_>,
	) -> async_graphql::Response {
		let resp = next.run(ctx).await;

		resp.errors.iter().for_each(handle_error);

		resp
	}

	async fn execute(
		&self,
		ctx: &async_graphql::extensions::ExtensionContext<'_>,
		operation_name: Option<&str>,
		next: async_graphql::extensions::NextExecute<'_>,
	) -> async_graphql::Response {
		let resp = next.run(ctx, operation_name).await;

		resp.errors.iter().for_each(handle_error);

		resp
	}

	async fn validation(
		&self,
		ctx: &async_graphql::extensions::ExtensionContext<'_>,
		next: async_graphql::extensions::NextValidation<'_>,
	) -> Result<async_graphql::ValidationResult, Vec<async_graphql::ServerError>> {
		next.run(ctx).await.inspect_err(|errors| {
			errors.iter().for_each(handle_error);
		})
	}
}
