use std::sync::Arc;

use shared::database::user::{FullUser, User};

use crate::{global::Global, http::error::ApiError};

#[derive(Debug, Clone)]
pub enum UserDataSource {
	User(User),
	Full(FullUser),
}

impl From<User> for UserDataSource {
	fn from(user: User) -> Self {
		Self::User(user)
	}
}

impl From<FullUser> for UserDataSource {
	fn from(user: FullUser) -> Self {
		Self::Full(user)
	}
}

impl UserDataSource {
	pub async fn full(&mut self, global: &Arc<Global>) -> Result<&FullUser, ApiError> {
		match self {
			UserDataSource::User(u) => {
				let full = global
					.user_loader()
					.load(global, u.id)
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
					.ok_or(ApiError::NOT_FOUND)?;
				*self = UserDataSource::Full(full);
				let full = if let UserDataSource::Full(full) = self {
					full
				} else {
					unreachable!()
				};
				Ok(full)
			}
			UserDataSource::Full(u) => Ok(u),
		}
	}

	pub fn user(&self) -> &User {
		match self {
			UserDataSource::User(u) => u,
			UserDataSource::Full(u) => &u.user,
		}
	}
}
