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
			Ok(Json(
				VALUE.get_or_init(|| os.info_for_version(&global, branch.resolve(&global))),
			))
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
	pub fn info_for_version(self, global: &Global, version: &str) -> serde_json::Value {
		// For backwards compatiblity:
		// Chatterino 7TV versions before 7.5.5 did not consider 7.x.y
		// to 2.x.(y+1) as an update, so we pretend the new version is actually
		// 7.x.(y+1).
		// For the newer versions, we include `v2_version` which contains the
		// actual version.
		let v7_version = if version.starts_with("2.") {
			Some(version.replacen("2.", "7.", 1))
		} else {
			None
		};
		let v7_version = v7_version.as_deref().unwrap_or(version);

		let experimental = if global.config.api.chatterino.windows_on_arm_is_experimental {
			"Experimental-"
		} else {
			""
		};

		let win_installer_x86 =
			format!("https://github.com/SevenTV/chatterino7/releases/download/v{version}/Chatterino7.Installer.zip");
		let win_portable_x86 =
			format!("https://github.com/SevenTV/chatterino7/releases/download/v{version}/Chatterino7.Portable.zip");
		let win_installer_arm = format!("https://github.com/SevenTV/chatterino7/releases/download/v{version}/{experimental}ARM64-Chatterino7.Installer.zip");
		let win_portable_arm = format!("https://github.com/SevenTV/chatterino7/releases/download/v{version}/{experimental}ARM64-Chatterino7.Portable.zip");
		match self {
			Os::Windows => serde_json::json!({
				"version": v7_version,
				"v2_version": version,
				"updateexe": win_installer_x86,
				"updateexe_x86": win_installer_x86,
				"updateexe_arm": win_installer_arm,
				"portable_download": win_portable_x86,
				"portable_download_x86": win_portable_x86,
				"portable_download_arm": win_portable_arm,
			}),
			Os::MacOs => serde_json::json!({
				"version": v7_version,
				"v2_version": version,
				"updateexe": format!("https://github.com/SevenTV/chatterino7/releases/download/v{version}/Chatterino.dmg"),
			}),
			Os::Linux => serde_json::json!({
				"version": v7_version,
				"v2_version": version,
				"updateguide": format!("https://github.com/SevenTV/chatterino7/releases/tag/v{version}"),
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
