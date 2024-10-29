use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct Image {
    pub url: String,
	pub mime: String,
	pub size: i64,
	pub scale: i32,
	pub width: i32,
	pub height: i32,
	pub frame_count: i32,
}

impl Image {
    pub fn from_db(value: shared::database::image_set::Image, cdn_base_url: &url::Url) -> Self {
        Self {
            url: value.get_url(cdn_base_url),
            mime: value.mime,
            size: value.size,
			scale: value.scale,
            width: value.width,
            height: value.height,
            frame_count: value.frame_count,
        }
    }
}
