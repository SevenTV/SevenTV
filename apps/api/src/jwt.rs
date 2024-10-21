use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use hmac::{Hmac, Mac};
use jwt_next::{Claims, Header, RegisteredClaims, SignWithKey, Token, VerifyWithKey};
use sha2::Sha256;
use shared::database::user::session::{UserSession, UserSessionId};
use shared::database::user::UserId;

use crate::global::Global;
use crate::http::middleware::session::Session;

pub struct AuthJwtPayload {
	pub user_id: UserId,
	pub session_id: UserSessionId,
	pub expiration: Option<DateTime<Utc>>,
	pub issued_at: DateTime<Utc>,
	pub not_before: Option<DateTime<Utc>>,
	pub audience: Option<String>,
}

pub trait JwtState: Sized {
	fn to_claims(&self) -> Claims;

	fn from_claims(claims: &Claims) -> Option<Self>;

	fn serialize(&self, global: &Arc<Global>) -> Option<String> {
		let config = global.config.jwt.clone();

		let key = Hmac::<Sha256>::new_from_slice(config.secret.as_bytes()).ok()?;
		let mut claims = self.to_claims();

		claims.registered.issuer = Some(config.issuer.clone());

		if claims.registered.issued_at.is_none() {
			claims.registered.issued_at = Some(chrono::Utc::now().timestamp() as u64);
		}

		claims.sign_with_key(&key).ok()
	}

	fn verify(global: &Arc<Global>, token: &str) -> Option<Self> {
		let config = global.config.jwt.clone();

		let key = Hmac::<Sha256>::new_from_slice(config.secret.as_bytes()).ok()?;
		let token: Token<Header, Claims, _> = token.verify_with_key(&key).ok()?;

		let claims = token.claims();

		if claims.registered.issuer.as_ref() != Some(&config.issuer) {
			return None;
		}

		let iat = Utc.timestamp_opt(claims.registered.issued_at? as i64, 0).single()?;
		if iat > Utc::now() {
			return None;
		}

		let nbf = claims
			.registered
			.not_before
			.and_then(|x| Utc.timestamp_opt(x as i64, 0).single());
		if let Some(nbf) = nbf {
			if nbf > Utc::now() {
				return None;
			}
		}

		let exp = claims
			.registered
			.expiration
			.and_then(|x| Utc.timestamp_opt(x as i64, 0).single());
		if let Some(exp) = exp {
			if exp < Utc::now() {
				return None;
			}
		}

		Self::from_claims(claims)
	}
}

impl JwtState for AuthJwtPayload {
	fn to_claims(&self) -> Claims {
		Claims {
			registered: RegisteredClaims {
				issuer: None,
				subject: Some(self.user_id.to_string()),
				audience: self.audience.clone(),
				expiration: self.expiration.map(|x| x.timestamp() as u64),
				not_before: self.not_before.map(|x| x.timestamp() as u64),
				issued_at: Some(self.issued_at.timestamp() as u64),
				json_web_token_id: Some(self.session_id.to_string()),
			},
			private: Default::default(),
		}
	}

	fn from_claims(claims: &Claims) -> Option<Self> {
		// We need to figure out if this is an old jwt token
		// Old token payloads look like this:
		// {
		//   "u": "63f927eff8070da4e44c018b",
		//   "v": 1,
		//   "iss": "seventv-api",
		//   "exp": 1724159847,
		//   "nbf": 1716383847,
		//   "iat": 1716383847
		// }
		// User id is encoded as object id in the `u` field and the session id is
		// missing.
		//
		// New tokens encode the user id as ULID in the subject field (`sub`) and the
		// session id as ULID in the jwt id (`jti`) field.

		// Old tokens don't have the `sub` field, so they will fail here
		let user_id = claims.registered.subject.as_ref().and_then(|x| x.parse().ok())?;

		Some(Self {
			audience: claims.registered.audience.clone(),
			expiration: claims
				.registered
				.expiration
				.and_then(|x| Utc.timestamp_opt(x as i64, 0).single()),
			issued_at: Utc.timestamp_opt(claims.registered.issued_at? as i64, 0).single()?,
			not_before: claims
				.registered
				.not_before
				.and_then(|x| Utc.timestamp_opt(x as i64, 0).single()),
			session_id: claims.registered.json_web_token_id.as_ref()?.parse().ok()?,
			user_id,
		})
	}
}

impl From<UserSession> for AuthJwtPayload {
	fn from(session: UserSession) -> Self {
		AuthJwtPayload {
			user_id: session.user_id,
			session_id: session.id,
			expiration: Some(session.expires_at),
			issued_at: session.id.timestamp(),
			not_before: None,
			audience: None,
		}
	}
}

pub struct CsrfJwtPayload {
	pub random: [u8; 32],
	pub user_id: Option<UserId>,
	pub _session_id: Option<UserSessionId>,
	pub expiration: DateTime<Utc>,
}

impl CsrfJwtPayload {
	pub fn new(session: &Session) -> Self {
		Self {
			random: rand::random(),
			_session_id: session.user_session().map(|s| s.id),
			user_id: session.user_id(),
			expiration: Utc::now() + chrono::Duration::minutes(5),
		}
	}

	pub fn random(&self) -> String {
		hex::encode(self.random)
	}

	pub fn validate_random(&self, random: &str) -> Option<bool> {
		let random: [u8; 32] = hex::decode(random).ok()?.try_into().ok()?;
		Some(random == self.random)
	}
}

impl JwtState for CsrfJwtPayload {
	fn to_claims(&self) -> Claims {
		Claims {
			registered: RegisteredClaims {
				issuer: None,
				subject: Some("csrf".to_string()),
				audience: self.user_id.map(|id| id.to_string()),
				expiration: Some(self.expiration.timestamp() as u64),
				not_before: None,
				issued_at: None,
				json_web_token_id: Some(self.random()),
			},
			private: Default::default(),
		}
	}

	fn from_claims(claims: &Claims) -> Option<Self> {
		Some(Self {
			expiration: Utc.timestamp_opt(claims.registered.expiration? as i64, 0).single()?,
			user_id: claims.registered.audience.as_ref().and_then(|x| x.parse().ok()),
			_session_id: None,

			random: hex::decode(claims.registered.json_web_token_id.as_ref()?)
				.ok()?
				.try_into()
				.ok()?,
		})
	}
}
