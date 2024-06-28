use std::sync::Arc;

use shared::database::user::{FullUser, User};

use crate::global::Global;
use crate::http::error::ApiError;

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
	pub async fn full(&mut self, global: &Arc<Global>) -> Result<Option<&FullUser>, ApiError> {
		match self {
			UserDataSource::User(u) => {
				let Some(full) = global
					.user_loader()
					.load(global, u.id)
					.await
					.map_err(|_| ApiError::INTERNAL_SERVER_ERROR)?
				else {
					return Ok(None);
				};
				*self = UserDataSource::Full(full);
				let full = if let UserDataSource::Full(full) = self {
					full
				} else {
					unreachable!()
				};
				Ok(Some(full))
			}
			UserDataSource::Full(u) => Ok(Some(u)),
		}
	}

	pub fn user(&self) -> &User {
		match self {
			UserDataSource::User(u) => u,
			UserDataSource::Full(u) => &u.user,
		}
	}
}
