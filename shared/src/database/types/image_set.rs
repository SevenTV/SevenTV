use chrono::format;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImageSet {
    pub input: ImageSetInput,
    pub outputs: Vec<Image>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ImageSetInput {
    Pending {
        path: String,
        mime: String,
        size: u64,
    },
    Image(Image),
}

impl ImageSetInput {
    pub fn is_pending(&self) -> bool {
        matches!(self, ImageSetInput::Pending { .. })
    }
}

impl Default for ImageSetInput {
    fn default() -> Self {
        ImageSetInput::Pending {
            path: String::default(),
            mime: String::default(),
            size: u64::default(),
        }
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub path: String,
    pub mime: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub frame_count: u32,
}

impl Image {
    pub fn get_url(&self, cdn_base_url: &str) -> String {
        format!("{}/{}", cdn_base_url, self.path)
    }
}
