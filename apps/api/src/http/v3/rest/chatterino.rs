use std::str::FromStr;
use std::sync::{Arc, OnceLock};

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::global::Global;
use crate::http::error::{ApiError, ApiErrorCode};
use crate::http::extract::Path;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_version_info))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/version/:os/:branch", get(get_version_info))
}

#[utoipa::path(
	get,
	path = "/v3/chatterino/version/{os}/{branch}",
	tag = "chatterino",
	responses(
		(status = 200, description = "Chatterino7 versions", body = ChatterinoVersion, content_type = "application/json"),
	),
	params(
		("os" = String, Path, description = "The name of the operating system (win/macos/linux)"), 
		("branch" = String, Path, description = "The update branch (stable/beta)"), 
	)
)]
#[tracing::instrument(skip_all)]
async fn get_version_info(
	State(global): State<Arc<Global>>,
	Path((os, branch)): Path<(String, String)>,
) -> Result<Json<&'static serde_json::Value>, ApiError> {
	let Ok(os) = os.parse::<Os>() else {
		return Err(ApiError::not_found(ApiErrorCode::BadRequest, "Unknown OS"));
	};
	let Ok(branch) = branch.parse::<Branch>() else {
		return Err(ApiError::not_found(ApiErrorCode::BadRequest, "Unknown branch"));
	};

	macro_rules! make_statics {
		() => {{
			static VALUE: OnceLock<serde_json::Value> = OnceLock::new();
			Ok(Json(VALUE.get_or_init(|| os.info_for_version(branch.resolve(&global)))))
		}};
	}

	match (os, branch) {
		(Os::Windows, Branch::Stable) => make_statics!(),
		(Os::Windows, Branch::Beta) => make_statics!(),
		(Os::MacOs, Branch::Stable) => make_statics!(),
		(Os::MacOs, Branch::Beta) => make_statics!(),
		(Os::Linux, Branch::Stable) => make_statics!(),
		(Os::Linux, Branch::Beta) => make_statics!(),
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Os {
	Windows,
	MacOs,
	Linux,
}

impl Os {
	pub fn info_for_version(self, version: &str) -> serde_json::Value {
		match self {
			Os::Windows => serde_json::json!({
				"version": version,
				"updateexe": format!(
					"https://github.com/SevenTV/chatterino7/releases/download/v{}/Chatterino7.Installer.exe", version
				),
				"portable_download": format!(
					"https://github.com/SevenTV/chatterino7/releases/download/v{}/Chatterino7.Portable.zip", version
				),
			}),
			Os::MacOs => serde_json::json!({
				"version": version,
				"updateexe": format!(
					"https://github.com/SevenTV/chatterino7/releases/download/v{}/Chatterino.dmg", version
				),
			}),
			Os::Linux => serde_json::json!({
				"version": version,
				"updateguide": format!(
					"https://github.com/SevenTV/chatterino7/releases/tag/v{}", version
				),
			}),
		}
	}
}

impl FromStr for Os {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"win" => Ok(Self::Windows),
			"macos" => Ok(Self::MacOs),
			"linux" => Ok(Self::Linux),
			_ => Err(()),
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Branch {
	Stable,
	Beta,
}

impl Branch {
	pub fn resolve(self, global: &Global) -> &str {
		match self {
			Branch::Stable => &global.config.api.chatterino.stable_version,
			Branch::Beta => global
				.config
				.api
				.chatterino
				.beta_version
				.as_deref()
				.unwrap_or(&global.config.api.chatterino.stable_version),
		}
	}
}

impl FromStr for Branch {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"stable" => Ok(Self::Stable),
			"beta" => Ok(Self::Beta),
			_ => Err(()),
		}
	}
}
