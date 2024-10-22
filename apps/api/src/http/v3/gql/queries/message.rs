use std::sync::Arc;

use async_graphql::{ComplexObject, Context, Enum, Object, SimpleObject};
use mongodb::bson::doc;
use shared::database::emote_moderation_request::{
	EmoteModerationRequest, EmoteModerationRequestKind, EmoteModerationRequestStatus,
};
use shared::database::role::permissions::EmoteModerationRequestPermission;
use shared::old_types::object_id::GqlObjectId;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::v3::gql::guards::{PermissionGuard, RateLimitGuard};
use crate::search::{search, sorted_results, SearchOptions};

// https://github.com/SevenTV/API/blob/main/internal/api/gql/v3/schema/messages.gql

#[derive(Default)]
pub struct MessagesQuery;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct InboxMessage {
	id: GqlObjectId,
	kind: MessageKind,
	created_at: chrono::DateTime<chrono::Utc>,
	author_id: Option<GqlObjectId>,
	read: bool,
	read_at: Option<chrono::DateTime<chrono::Utc>>,
	subject: String,
	content: String,
	important: bool,
	starred: bool,
	pinned: bool,
	placeholders: StringMap,
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex, rename_fields = "snake_case")]
pub struct ModRequestMessage {
	id: GqlObjectId,
	kind: MessageKind,
	// created_at
	author_id: Option<GqlObjectId>,
	read: bool,
	read_at: Option<chrono::DateTime<chrono::Utc>>,
	target_kind: u32,
	target_id: GqlObjectId,
	wish: String,
	actor_country_name: String,
	actor_country_code: String,
}

impl ModRequestMessage {
	fn from_db(mod_request: EmoteModerationRequest) -> Self {
		let country = mod_request.country_code.unwrap_or_default();

		Self {
			id: mod_request.id.into(),
			kind: MessageKind::ModRequest,
			author_id: Some(mod_request.user_id.into()),
			read: mod_request.status == EmoteModerationRequestStatus::Approved
				|| mod_request.status == EmoteModerationRequestStatus::Denied,
			read_at: None,
			target_kind: 2,
			target_id: mod_request.emote_id.into(),
			wish: match mod_request.kind {
				EmoteModerationRequestKind::PublicListing => "list".to_string(),
				EmoteModerationRequestKind::PersonalUse => "personal_use".to_string(),
			},
			actor_country_name: country.clone(),
			actor_country_code: country,
		}
	}
}

#[ComplexObject(rename_fields = "snake_case", rename_args = "snake_case")]
impl ModRequestMessage {
	async fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
		self.id.0.timestamp()
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
#[graphql(rename_items = "SCREAMING_SNAKE_CASE")]
pub enum MessageKind {
	EmoteComment,
	ModRequest,
	Inbox,
	News,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StringMap(async_graphql::indexmap::IndexMap<String, String>);

async_graphql::scalar!(StringMap);

#[derive(Debug, Clone, Default, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct ModRequestMessageList {
	messages: Vec<ModRequestMessage>,
	total: u64,
}

#[Object(rename_fields = "camelCase", rename_args = "snake_case")]
impl MessagesQuery {
	async fn announcement<'ctx>(&self, ctx: &Context<'ctx>) -> Result<String, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let message = global
			.global_config_loader
			.load(())
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load global config"))?
			.ok_or_else(|| ApiError::internal_server_error(ApiErrorCode::LoadError, "global config not found"))?
			.alerts
			.message;

		Ok(message.unwrap_or_default())
	}

	async fn inbox<'ctx>(&self) -> Vec<InboxMessage> {
		// not implemented
		vec![]
	}

	#[graphql(guard = "PermissionGuard::one(EmoteModerationRequestPermission::Manage).and(RateLimitGuard::search(1))")]
	async fn mod_requests<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(maximum = 50))] page: Option<u32>,
		#[graphql(validator(maximum = 250))] limit: Option<u32>,
		#[graphql(validator(max_length = 100))] wish: Option<String>,
		#[graphql(validator(max_items = 100))] country: Option<Vec<String>>,
	) -> Result<ModRequestMessageList, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let wish = wish
			.map(|w| match w.as_ref() {
				"personal_use" => EmoteModerationRequestKind::PersonalUse,
				_ => EmoteModerationRequestKind::PublicListing,
			})
			.unwrap_or(EmoteModerationRequestKind::PublicListing);

		let mut filters = vec![
			format!("kind: {}", wish as i32),
			format!("status: {}", EmoteModerationRequestStatus::Pending as i32),
		];

		let mut country_filters = vec![];

		for country in country.unwrap_or_default() {
			if country.len() > 100 {
				return Err(ApiError::bad_request(ApiErrorCode::BadRequest, "country code is too long"));
			}

			let sanitized = country.replace('`', "");
			country_filters.push(format!("country_code: {}", sanitized.trim_end_matches('\\')));
		}

		if !country_filters.is_empty() {
			filters.push(format!("({})", country_filters.join(" || ")));
		}

		let options = SearchOptions::builder()
			.query("*".to_owned())
			.filter_by(filters.join(" && "))
			.sort_by(vec!["priority:desc".to_owned(), "created_at:asc".to_owned()])
			.page(page)
			.per_page(limit)
			.build();

		let result = search::<shared::typesense::types::emote_moderation_request::EmoteModerationRequest>(global, options)
			.await
			.map_err(|err| {
				tracing::error!(error = %err, "failed to search");
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to search")
			})?;

		let requests = global
			.emote_moderation_request_by_id_loader
			.load_many(result.hits.iter().copied())
			.await
			.map_err(|()| {
				ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load emote moderation requests")
			})?;

		Ok(ModRequestMessageList {
			messages: sorted_results(result.hits, requests)
				.into_iter()
				.map(ModRequestMessage::from_db)
				.collect(),
			total: result.found,
		})
	}
}
