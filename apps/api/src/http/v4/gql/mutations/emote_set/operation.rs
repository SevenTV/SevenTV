use shared::database::user::FullUserRef;

use crate::http::{middleware::session::Session, v4::gql::types::EmoteSetEmoteFlags};

#[async_graphql::SimpleObject]
#[graphql(complex)]
pub struct EmoteSetOperation {
	#[graphql(skip)]
	pub emote_set: shared::database::emote_set::EmoteSet,
}

impl EmoteSetOperation {
	async fn check_perms<'a>(
		&self,
		global: &Arc<Global>,
		session: &'a Session,
		editor_perm: impl Into<EditorPermission>,
	) -> Result<FullUserRef<'a>, ApiError> {
		let mut editor_perm = editor_perm.into();
		let user = session.user()?;

		let mut target = FullUserRef::Ref(user);

		match self.emote_set.kind {
			EmoteSetKind::Global => {
				if !user.has(EmoteSetPermission::ManageGlobal) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"this user does not have permission to manage global emote sets",
					));
				}

				return Ok(target);
			}
			EmoteSetKind::Special => {
				if !user.has(EmoteSetPermission::ManageSpecial) {
					return Err(ApiError::forbidden(
						ApiErrorCode::LackingPrivileges,
						"this user does not have permission to manage special emote sets",
					));
				}

				return Ok(target);
			}
			EmoteSetKind::Personal | EmoteSetKind::Normal => {}
		}

		let owner_id = self
			.emote_set
			.owner_id
			.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "owner not found"))?;

		// If the person who is updating the set is not the owner, we need to load the
		// owner.
		if owner_id != user.id {
			target = FullUserRef::Owned(
				global
					.user_loader
					.load(global, owner_id)
					.await
					.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load user"))?
					.ok_or_else(|| ApiError::not_found(ApiErrorCode::LoadError, "owner not found"))?
					.into(),
			)
		}

		let mut forbidden_msg = "you do not have permission to manage this user's emote sets";

		// If the emote set is personal, check if the owner has permission to use
		if matches!(self.emote_set.kind, EmoteSetKind::Personal) {
			if !target.has(UserPermission::UsePersonalEmoteSet) {
				return Err(ApiError::forbidden(
					ApiErrorCode::LackingPrivileges,
					"this user does not have permission to use personal emote sets",
				));
			}

			editor_perm = EditorUserPermission::ManagePersonalEmoteSet.into();
			forbidden_msg = "you do not have permission to manage this user's personal emote set";
		}

		if !target.has(EmoteSetPermission::Manage) && !user.has(EmoteSetPermission::ManageAny) {
			return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, forbidden_msg));
		}

		if target.id != user.id && !user.has(EmoteSetPermission::ManageAny) {
			let editor = global
				.user_editor_by_id_loader
				.load(UserEditorId {
					user_id: owner_id,
					editor_id: user.id,
				})
				.await
				.map_err(|()| ApiError::internal_server_error(ApiErrorCode::LoadError, "failed to load editor"))?
				.ok_or_else(|| {
					ApiError::forbidden(ApiErrorCode::LackingPrivileges, "you are not an editor for this user")
				})?;

			if editor.state != UserEditorState::Accepted || !editor.permissions.has(editor_perm) {
				return Err(ApiError::forbidden(ApiErrorCode::LackingPrivileges, forbidden_msg));
			}
		}

		Ok(target)
	}
}

#[derive(async_graphql::InputObject)]
pub struct EmoteSetEmoteId {
	pub emote_id: EmoteId,
	pub alias: Option<String>,
}

#[derive(async_graphql::InputObject)]
pub struct AddEmote {
	pub id: EmoteSetEmoteId,
	pub zero_width: Option<bool>,
	pub override_conflicts: Option<bool>,
}

#[async_graphql::ComplexObject]
impl EmoteSetOperation {
	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetCreate, 1))"
	)]
	async fn create(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(custom = "NameValidator"))] name: String,
	) -> Result<EmoteSet, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn name(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(custom = "NameValidator"))] name: String,
	) -> Result<EmoteSet, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn capacity(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(minimum = 1))] capacity: i32,
	) -> Result<EmoteSet, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn add_emotes(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(min_items = 1, max_items = 50))] emotes: Vec<AddEmote>,
	) -> Result<EmoteSet, ApiError> {
		let global: &Arc<Global> = ctx
			.data()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing global data"))?;
		let session = ctx
			.data::<Session>()
			.map_err(|_| ApiError::internal_server_error(ApiErrorCode::MissingContext, "missing sesion data"))?;

		self.check_perms(global, session, EditorEmoteSetPermission::Manage).await?;

		let res = transaction_with_mutex(
			global,
			Some(GeneralMutexKey::EmoteSet(self.emote_set.id).into()),
			|tx| async move {
				Ok(())
			},
		)
		.await;

		match res {
			Ok(emote_set) => todo!(),
			Err(TransactionError::Custom(e)) => Err(e),
			Err(e) => {
				tracing::error!(error = %e, "transaction failed");
				Err(ApiError::internal_server_error(
					ApiErrorCode::TransactionError,
					"transaction failed",
				))
			}
		}
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn remove_emotes(
		&self,
		ctx: &Context<'ctx>,
		#[graphql(validator(min_items = 1, max_items = 50))] emotes: Vec<EmoteSetEmoteId>,
	) -> Result<EmoteSet, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn update_emote_alias(
		&self,
		ctx: &Context<'ctx>,
		id: EmoteSetEmoteId,
		#[graphql(validator(custom = "EmoteNameValidator"))] alias: String,
	) -> Result<EmoteSetEmote, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn update_emote_flags(
		&self,
		ctx: &Context<'ctx>,
		id: EmoteSetEmoteId,
		flags: EmoteSetEmoteFlags,
	) -> Result<EmoteSetEmote, ApiError> {
		todo!()
	}

	#[graphql(
		guard = "PermissionGuard::one(EmoteSetPermission::Manage).and(RateLimitGuard::new(RateLimitResource::EmoteSetChange, 1))"
	)]
	async fn delete<'ctx>(&self, ctx: &Context<'ctx>) -> Result<bool, ApiError> {
		todo!()
	}
}
