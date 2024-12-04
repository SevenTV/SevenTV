use crate::http::error::{ApiError, ApiErrorCode};

#[derive(Default)]
pub struct BackdoorQuery;

#[async_graphql::Object]
impl BackdoorQuery {
	#[tracing::instrument(skip_all, name = "BackdoorQuery::execute_sql")]
	async fn execute_sql(&self, _sql: String) -> Result<String, ApiError> {
		// TODO
		Err(ApiError::not_implemented(
			ApiErrorCode::Unknown,
			"backdoor not implemented yet",
		))
	}
}
