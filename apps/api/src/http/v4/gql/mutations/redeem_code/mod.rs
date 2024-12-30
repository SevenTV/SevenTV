use std::sync::Arc;

use async_graphql::Context;
use rand::distributions::DistString;
use shared::database::product::codes::RedeemCodeId;
use shared::database::product::special_event::SpecialEventId;
use shared::database::product::SubscriptionProductId;
use shared::database::role::permissions::UserPermission;
use shared::database::MongoCollection;

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::guards::PermissionGuard;
use crate::http::middleware::session::Session;
use crate::http::v4::gql::types::{RedeemCode, TimePeriodInput};

mod operation;

#[derive(Default)]
pub struct RedeemCodeMutation;

#[derive(async_graphql::InputObject)]
struct CreateRedeemCodeInput {
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: Option<String>,
	pub uses: u32,
	pub active_period: Option<TimePeriodInput>,
	pub special_event_id: SpecialEventId,
	pub subscription_effect: Option<RedeemCodeSubscriptionEffectInput>,
}

#[derive(async_graphql::InputObject)]
struct CreateRedeemCodeBatchInput {
	pub number: u32,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub uses: u32,
	pub active_period: Option<TimePeriodInput>,
	pub special_event_id: SpecialEventId,
	pub subscription_effect: Option<RedeemCodeSubscriptionEffectInput>,
}

#[derive(async_graphql::InputObject)]
struct RedeemCodeSubscriptionEffectInput {
	pub product_id: SubscriptionProductId,
	pub trial_days: Option<u32>,
	pub no_no_redirect_to_stripe: bool,
}

impl From<RedeemCodeSubscriptionEffectInput> for shared::database::product::codes::RedeemCodeSubscriptionEffect {
	fn from(input: RedeemCodeSubscriptionEffectInput) -> Self {
		Self {
			id: input.product_id,
			trial_days: input.trial_days.map(|d| d as i32),
			no_redirect_to_stripe: input.no_no_redirect_to_stripe,
		}
	}
}

fn generate_code(len: usize) -> String {
	rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}

#[async_graphql::Object]
impl RedeemCodeMutation {
	#[tracing::instrument(skip_all, name = "RedeemCodeMutation::redeem_code")]
	async fn redeem_code(&self, ctx: &Context<'_>, id: RedeemCodeId) -> Result<operation::RedeemCodeOperation, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;

		let code = global
			.redeem_code_by_id_loader
			.load(id)
			.await
			.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "user not found"))?;

		Ok(operation::RedeemCodeOperation { code })
	}

	#[tracing::instrument(skip_all, name = "RedeemCodeMutation::create")]
	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageBilling)")]
	async fn create(&self, ctx: &Context<'_>, data: CreateRedeemCodeInput) -> Result<RedeemCode, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let created_by = session.user()?;

		let code = data.code.unwrap_or_else(|| generate_code(6));

		let redeem_code = shared::database::product::codes::RedeemCode {
			id: RedeemCodeId::default(),
			name: data.name,
			description: data.description,
			tags: data.tags,
			code,
			remaining_uses: data.uses as i32,
			active_period: data.active_period.map(Into::into),
			effect: shared::database::product::codes::CodeEffect::SpecialEvent {
				special_event_id: data.special_event_id,
			},
			subscription_effect: data.subscription_effect.map(Into::into),
			created_by: created_by.id,
			updated_at: chrono::Utc::now(),
			search_updated_at: None,
		};

		shared::database::product::codes::RedeemCode::collection(&global.db)
			.insert_one(&redeem_code)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create redeem code");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to create redeem code")
			})?;

		Ok(redeem_code.into())
	}

	#[tracing::instrument(skip_all, name = "RedeemCodeMutation::create")]
	#[graphql(guard = "PermissionGuard::one(UserPermission::ManageBilling)")]
	async fn create_batch(&self, ctx: &Context<'_>, data: CreateRedeemCodeBatchInput) -> Result<Vec<RedeemCode>, ApiError> {
		let global = ctx
			.data::<Arc<Global>>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing session data"))?;
		let created_by = session.user()?;
		let active_period = data.active_period.map(Into::into);
		let subscription_effect = data.subscription_effect.map(Into::into);

		let mut redeem_codes = Vec::new();

		for _ in 0..data.number {
			let code = generate_code(6);

			redeem_codes.push(shared::database::product::codes::RedeemCode {
				id: RedeemCodeId::default(),
				name: data.name.clone(),
				description: data.description.clone(),
				tags: data.tags.clone(),
				code,
				remaining_uses: data.uses as i32,
				active_period,
				effect: shared::database::product::codes::CodeEffect::SpecialEvent {
					special_event_id: data.special_event_id,
				},
				subscription_effect: subscription_effect.clone(),
				created_by: created_by.id,
				updated_at: chrono::Utc::now(),
				search_updated_at: None,
			});
		}

		shared::database::product::codes::RedeemCode::collection(&global.db)
			.insert_many(&redeem_codes)
			.await
			.map_err(|e| {
				tracing::error!(error = %e, "failed to create redeem codes");
				ApiError::internal_server_error(ApiErrorCode::MutationError, "failed to create redeem codes")
			})?;

		Ok(redeem_codes.into_iter().map(Into::into).collect())
	}
}
