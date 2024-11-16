mod operation;

#[derive(Default)]
pub struct EmoteSetMutation;

#[async_graphql::Object]
impl EmoteSetMutation {
	async fn emote_set<'ctx>(&self, ctx: &Context<'ctx>, id: EmoteSetId) -> Result<operation::EmoteSetOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let emote_set = global
			.emote_set_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote set"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "emote set not found"))?;

		Ok(operation::EmoteSetOperation { emote_set })
	}
}
