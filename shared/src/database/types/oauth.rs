use rand::Rng;

use super::user::UserId;
use super::MongoGenericCollection;
use crate::database::{Id, MongoCollection};

pub type OAuthAppId = Id<OAuthApp>;

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "oauth_apps")]
#[serde(deny_unknown_fields)]
pub struct OAuthApp {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: OAuthAppId,
	pub name: String,
	pub description: Option<String>,
	#[serde(with = "crate::database::serde")]
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "oauth_app_users")]
#[serde(deny_unknown_fields)]
pub struct OAuthAppUser {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: OAuthAppUserId,
	pub linked_by: UserId,
	pub linked_at: chrono::DateTime<chrono::Utc>,
	pub scopes: Vec<OAuthUserScope>,
	pub created_by_grant: OAuthAppTokenId,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Hash, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OAuthAppUserId {
	pub app_id: OAuthAppId,
	pub user_id: UserId,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "oauth_app_user_flows")]
pub struct OAuthAppUserFlow {
	#[mongo(id)]
	#[serde(rename = "_id")]
	/// This is the code generated to complete the oauth flow
	pub id: OAuthToken,
	pub app_id: OAuthAppId,
	pub user_id: UserId,
	pub scopes: Vec<OAuthUserScope>,
	pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum OAuthUserScope {}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum OAuthAppScope {
	User(OAuthUserScope),
}

pub type OAuthImplicitUserTokenId = Id<OAuthImplicitUserToken>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "oauth_user_tokens")]
pub struct OAuthImplicitUserToken {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: OAuthImplicitUserTokenId,
	pub app_user_id: OAuthAppUserId,
	pub access_token: OAuthToken,
	pub refresh_token: OAuthToken,
	pub access_token_expires_at: chrono::DateTime<chrono::Utc>,
	pub refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
	pub scopes: Vec<OAuthUserScope>,
	pub ip_ranges: Vec<ipnet::IpNet>,
	pub origins: Vec<String>,
	pub user_agent: Option<String>,
}

pub type OAuthAppTokenId = Id<OAuthAppToken>;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, MongoCollection)]
#[mongo(collection_name = "oauth_app_tokens")]
pub struct OAuthAppToken {
	#[mongo(id)]
	#[serde(rename = "_id")]
	pub id: OAuthAppTokenId,
	pub app_id: OAuthAppId,
	/// This is the token that will be used to perform api requests
	pub access_token: OAuthToken,
	/// If the access token expires, this token can be used to get a new access
	/// token
	pub refresh_token: Option<OAuthToken>,
	/// The time at which the access token expires
	pub access_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
	/// The time at which the refresh token expires
	pub refresh_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
	/// The scopes that this token has access to
	pub scopes: Vec<OAuthAppScope>,
	/// The ip ranges that this token can be used from (if empty, no
	/// restrictions)
	pub ip_restrictions: Vec<ipnet::IpNet>,
	/// The user agent that this token can be used from (if empty, no
	/// restrictions)
	pub user_agent: Option<String>,
	pub disabled: bool,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OAuthToken(u128);

impl OAuthToken {
	pub fn new() -> Self {
		let mut rng = rand::thread_rng();
		Self(rng.gen())
	}

	pub fn to_string(&self) -> String {
		format!("{:032x}", self.0)
	}

	pub fn from_string(s: &str) -> Option<Self> {
		u128::from_str_radix(s, 16).ok().map(Self)
	}
}

impl std::fmt::Display for OAuthToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:032x}", self.0)
	}
}

impl std::str::FromStr for OAuthToken {
	type Err = <u128 as std::str::FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		u128::from_str_radix(s, 16).map(Self)
	}
}

fn is_bson_ser<U>() -> bool {
	std::any::type_name::<U>().contains("bson::ser")
}

fn is_bson_de<U>() -> bool {
	std::any::type_name::<U>().contains("bson::de")
}

impl serde::Serialize for OAuthToken {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		if is_bson_ser::<S>() {
			serde::Serialize::serialize(&bson::Uuid::from_bytes(self.0.to_be_bytes()), serializer)
		} else {
			serde::Serialize::serialize(&self.to_string(), serializer)
		}
	}
}

impl<'de> serde::Deserialize<'de> for OAuthToken {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		if is_bson_de::<D>() {
			let uuid = bson::Uuid::deserialize(deserializer)?;
			Ok(Self(u128::from_be_bytes(uuid.bytes())))
		} else {
			let s = String::deserialize(deserializer)?;
			Self::from_string(&s).ok_or_else(|| serde::de::Error::custom("invalid OAuth token"))
		}
	}
}

impl std::fmt::Debug for OAuthToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "OAuth(<redacted>)")
	}
}

pub(super) fn mongo_collections() -> impl IntoIterator<Item = MongoGenericCollection> {
	[
		MongoGenericCollection::new::<OAuthApp>(),
		MongoGenericCollection::new::<OAuthGrant>(),
	]
}
