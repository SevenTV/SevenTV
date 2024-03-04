use postgres_types::{FromSql, ToSql};

mod association_badge;
mod association_emote_set;
mod association_paint;
mod association_product;
mod association_role;
mod code;
mod code_association_product;
mod purchase;
mod subscription;

pub use association_badge::*;
pub use association_emote_set::*;
pub use association_paint::*;
pub use association_product::*;
pub use association_role::*;
pub use code::*;
pub use code_association_product::*;
pub use purchase::*;
pub use subscription::*;

#[derive(Debug, Clone, Default, postgres_from_row::FromRow)]
pub struct Product {
	pub id: ulid::Ulid,
	pub name: String,
	pub description: String,
	pub tags: Vec<String>,
	pub enabled: bool,
	pub remaining_stock: Option<i32>,
	pub kind: ProductKind,
	pub rank: i16,
	pub visibility: ProductVisibility,
	pub data: ProductData,
	pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ProductData {}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_kind")]
pub enum ProductKind {
	#[default]
	#[postgres(name = "BASE")]
	Base,
	#[postgres(name = "ADDON")]
	Addon,
	#[postgres(name = "BUNDLE")]
	Bundle,
}

#[derive(Debug, Clone, Default, ToSql, FromSql)]
#[postgres(name = "product_visibility")]
pub enum ProductVisibility {
	#[default]
	#[postgres(name = "PUBLIC")]
	Public,
	#[postgres(name = "UNLISTED")]
	Unlisted,
}
