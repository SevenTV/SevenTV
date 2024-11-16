#[async_graphql::SimpleObject]
#[graphql(complex)]
pub struct EmoteOperation {
	#[graphql(skip)]
	pub emote: shared::database::emote::Emote,
}

#[async_graphql::ComplexObject]
impl EmoteOperation {
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn name(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(custom = "EmoteNameValidator"))] name: String,
	) -> Result<Emote, ApiError> {

	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn flags(
		&self,
		ctx: &Context<'ctx>,
		flags: EmoteFlags,
	) -> Result<Emote, ApiError> {

	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn owner(
		&self,
		ctx: &Context<'ctx>,
		owner_id: UserId,
	) -> Result<Emote, ApiError> {

	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn tags(
		&self,
		ctx: &Context<'ctx>,
		tags: Vec<String>,
	) -> Result<Emote, ApiError> {

	}

	#[graphql(guard = "PermissionGuard::one(EmotePermission::Merge)")]
	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn merge(
		&self,
		ctx: &Context<'ctx>,
		with: EmoteId,
	) -> Result<Emote, ApiError> {

	}

	#[graphql(guard = "RateLimitGuard::new(RateLimitResource::EmoteUpdate, 1)")]
	async fn delete(
		&self,
		ctx: &Context<'ctx>,
		reason: Option<String>,
	) -> Result<Emote, ApiError> {

	}
}
