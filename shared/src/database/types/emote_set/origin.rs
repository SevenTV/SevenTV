use super::EmoteSetId;
use crate::database::emote::EmoteId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
#[serde(deny_unknown_fields)]
pub enum EmoteSetOriginError {
	CycleDetected(String),
	MaxDepthExceeded(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetOriginConfig {
	pub origins: Vec<EmoteSetOrigin>,
	// Specify a limit on the number of emotes imported.
	pub limit: usize,
	// A list of emotes that are removed from every upstream set before computing the final set.
	pub purge: Vec<EmoteSetEmoteRef>,
	pub auto_resync: bool,
	pub needs_resync: bool,
	// Cycle was detected in the origin configuration, on this emote set id.
	pub error: Option<EmoteSetOriginError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "value")]
#[serde(deny_unknown_fields)]
pub enum EmoteSetEmoteRef {
	Alias(String),
	Id(EmoteId),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetOrigin {
	pub id: EmoteSetId,
	pub limit: Option<EmoteSetLimit>,
	pub transformations: Vec<EmoteSetOriginTransformation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmoteSetLimit {
	pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum EmoteSetOriginTransformation {
	Exclude { emote: EmoteSetEmoteRef },
	Rename { old_alias: String, new_alias: String },
}
