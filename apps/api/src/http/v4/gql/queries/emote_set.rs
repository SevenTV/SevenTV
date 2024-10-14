use async_graphql::Object;
use shared::database::emote_set::EmoteSetId;

use crate::http::v4::gql::types::EmoteSet;

#[derive(Default)]
pub struct EmoteSetQuery;

#[Object]
impl EmoteSetQuery {
	async fn emote_set(&self, _id: EmoteSetId) -> Option<EmoteSet> {
		None
	}
}
