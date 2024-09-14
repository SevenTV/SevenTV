use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct Image {
    pub path: String,
	pub mime: String,
	pub size: i64,
	pub width: i32,
	pub height: i32,
	pub frame_count: i32,
}

impl for Image {
    pub fn from_db(value: shared::database::image_set::Image, cdn_bas_url: &url::Url) -> Self {
        Self {
            path: value.path,
            mime: value.mime,
            size: value.size,
            width: value.width,
            height: value.height,
            frame_count: value.frame_count,
        }
    }
}
