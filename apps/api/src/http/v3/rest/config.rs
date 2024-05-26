use std::sync::{Arc, OnceLock};

use axum::routing::get;
use axum::{Json, Router};

use crate::global::Global;
use crate::http::error::ApiError;
use crate::http::extract::Path;

#[derive(utoipa::OpenApi)]
#[openapi(paths(get_extension_config))]
pub struct Docs;

pub fn routes() -> Router<Arc<Global>> {
	Router::new().route("/:name", get(get_extension_config))
}

static EXTENSION: OnceLock<serde_json::Value> = OnceLock::new();

static EXTENSION_NIGHTLY: OnceLock<serde_json::Value> = OnceLock::new();

#[utoipa::path(
    get,
    path = "/v3/config/{name}",
    tag = "config",
    responses(
        (status = 200, description = "Extension Config", body = ExtensionConfig, content_type = "application/json"),
    ),
    params(
        ("name" = String, Path, description = "The name of the extension to get the config for"),
    )
)]
#[tracing::instrument]
// https://github.com/SevenTV/API/blob/c47b8c8d4f5c941bb99ef4d1cfb18d0dafc65b97/internal/api/rest/v3/routes/config/config.root.go#L29
async fn get_extension_config(Path(name): Path<String>) -> Result<Json<&'static serde_json::Value>, ApiError> {
	// Hard coded response because this will be deprecated
	if name == "extension" {
		Ok(Json(EXTENSION.get_or_init(|| serde_json::json!({"version":"3.0.9","overrides":[],"compatibility":[{"id":["ajopnjidmegmdimjlfnijceegpefgped","firefox@betterttv.net"],"issues":[{"platform":null,"severity":"NOTE","message":"7TV supports BetterTTV emotes by default"},{"platform":null,"severity":"DUPLICATE_FUNCTIONALITY","message":"A significant amount of functionality from this extension is also in 7TV. While there is no known immediate clashing with this extension, such redundancy may decrease performance"}]},{"id":["fadndhdgpmmaapbmfcknlfgcflmmmieb","frankerfacez@frankerfacez.com"],"issues":[{"platform":null,"severity":"NOTE","message":"7TV supports FrankerFaceZ emotes by default. This extension is compatibility-tested by our team. However, many of its features may currently be unavailable. We are working on improving integration."}]},{"id":["fooolghllnmhmmndgjiamiiodkpenpbb"],"issues":[{"platform":null,"severity":"BAD_PERFORMANCE","message":"This extension attempts to hook the chat input box, which is partially prevented by 7TV. This leads to severe lag spikes. Please consider switching to a different password manager"},{"platform":null,"severity":"WARNING","message":"Our analysis of this extension revealed that it's highly obfuscated and appears to be extremely inefficient"}]},{"id":["ipnllhnoiiclnoonckahfcpahgehgdgb"],"issues":[{"platform":null,"severity":"CLASHING","message":"This extension is wholly incompatible with 7TV's high-performance chat and breaks layout"}]},{"id":["bhplkbgoehhhddaoolmakpocnenplmhf","twitch5@coolcmd"],"issues":[{"platform":null,"severity":"WARNING","message":"7TV cannot function with this extension because it runs inside its own isolated context. Please disable it."}]}]}))))
	} else if name == "extension-nightly" {
		Ok(Json(EXTENSION_NIGHTLY.get_or_init(|| serde_json::json!({"version":"3.0.10.1000","overrides":[],"compatibility":[{"id":["ajopnjidmegmdimjlfnijceegpefgped","firefox@betterttv.net"],"issues":[{"platform":null,"severity":"NOTE","message":"7TV supports BetterTTV emotes by default"},{"platform":null,"severity":"DUPLICATE_FUNCTIONALITY","message":"A significant amount of functionality from this extension is also in 7TV. While there is no known immediate clashing with this extension, such redundancy may decrease performance"}]},{"id":["fadndhdgpmmaapbmfcknlfgcflmmmieb","frankerfacez@frankerfacez.com"],"issues":[{"platform":null,"severity":"NOTE","message":"7TV supports FrankerFaceZ emotes by default. This extension is compatibility-tested by our team. However, many of its features may currently be unavailable. We are working on improving integration."},{"platform":null,"severity":"WARNING","message":"There may currently be some odd behaviors when using this extension alongside 7TV whilst acting as a Moderator"}]},{"id":["fooolghllnmhmmndgjiamiiodkpenpbb"],"issues":[{"platform":null,"severity":"BAD_PERFORMANCE","message":"This extension attempts to hook the chat input box, which is partially prevented by 7TV. This leads to severe lag spikes. Please consider switching to a different password manager"},{"platform":null,"severity":"WARNING","message":"Our analysis of this extension revealed that it's highly obfuscated and appears to be extremely inefficient"}]},{"id":["ipnllhnoiiclnoonckahfcpahgehgdgb"],"issues":[{"platform":null,"severity":"CLASHING","message":"This extension is wholly incompatible with 7TV's high-performance chat and breaks layout"}]},{"id":["bhplkbgoehhhddaoolmakpocnenplmhf","twitch5@coolcmd"],"issues":[{"platform":null,"severity":"WARNING","message":"7TV cannot function with this extension because it runs inside its own isolated context. Please disable it."}]}]}))))
	} else {
		Err(ApiError::NOT_FOUND)
	}
}
