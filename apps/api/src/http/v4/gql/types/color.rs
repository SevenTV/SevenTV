use async_graphql::Object;

#[derive(Debug, Copy, Clone)]
pub struct Color(pub i32);

impl From<i32> for Color {
	fn from(value: i32) -> Self {
		Self(value)
	}
}

#[Object]
impl Color {
	#[tracing::instrument(skip_all, name = "Color::hex")]
	pub async fn hex(&self) -> String {
		format!("#{:08X}", self.0)
	}

	#[tracing::instrument(skip_all, name = "Color::r")]
	pub async fn r(&self) -> u8 {
		((self.0 >> 24) & 0xFF) as u8
	}

	#[tracing::instrument(skip_all, name = "Color::g")]
	pub async fn g(&self) -> u8 {
		((self.0 >> 16) & 0xFF) as u8
	}

	#[tracing::instrument(skip_all, name = "Color::b")]
	pub async fn b(&self) -> u8 {
		((self.0 >> 8) & 0xFF) as u8
	}

	#[tracing::instrument(skip_all, name = "Color::a")]
	pub async fn a(&self) -> u8 {
		(self.0 & 0xFF) as u8
	}
}
