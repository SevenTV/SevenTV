#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSet {
	pub input: ImageSetInput,
	pub outputs: Vec<Image>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
pub enum ImageSetInput {
	Pending {
		task_id: String,
		path: String,
		mime: String,
		size: i64,
	},
	Image(Image),
}

impl ImageSetInput {
	pub fn is_pending(&self) -> bool {
		matches!(self, ImageSetInput::Pending { .. })
	}
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
	pub path: String,
	pub mime: String,
	pub size: i64,
	pub width: i32,
	pub height: i32,
	pub frame_count: i32,
}

impl Image {
	pub fn get_url(&self, cdn_base_url: &str) -> String {
		format!("{}/{}", cdn_base_url, self.path)
	}
}
