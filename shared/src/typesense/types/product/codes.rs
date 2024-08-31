use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::database::product::codes::{DiscountCodeId, RedeemCodeId, SpecialEventId};
use crate::database::product::ProductId;
use crate::database::user::UserId;
use crate::typesense::types::{impl_typesense_type, TypesenseCollection, TypesenseGenericCollection};

#[derive(Debug, Clone, Serialize, Deserialize, TypesenseCollection)]
#[typesense(collection_name = "discount_codes")]
#[serde(deny_unknown_fields)]
pub struct DiscountCode {
	pub id: DiscountCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub discount_kind: DiscountKind,
	pub discount_amount: f64,
	pub product_ids: Vec<ProductId>,
	/// Set when fixed discount
	#[typesense(optional, field = "string")]
	pub discount_currency: Option<stripe::Currency>,
	pub active_from: i64,
	pub active_to: i64,
	pub max_uses: Option<i32>,
	pub remaining_uses: Option<i32>,
	pub max_uses_per_user: Option<i32>,
	pub created_by: UserId,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

#[derive(Debug, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(i32)]
pub enum DiscountKind {
	Percentage = 0,
	Fixed = 1,
}

impl DiscountKind {
	fn split(discount: database::product::codes::Discount) -> (DiscountKind, f64, Option<stripe::Currency>) {
		match discount {
			database::product::codes::Discount::Percentage { percentage } => (DiscountKind::Percentage, percentage, None),
			database::product::codes::Discount::Fixed { amount, currency } => (DiscountKind::Fixed, amount, Some(currency)),
		}
	}
}

impl_typesense_type!(DiscountKind, Int32);

impl From<database::product::codes::DiscountCode> for DiscountCode {
	fn from(discount: database::product::codes::DiscountCode) -> Self {
		let (discount_kind, discount_amount, discount_currency) = DiscountKind::split(discount.discount);

		Self {
			id: discount.id,
			name: discount.name,
			description: discount.description,
			tags: discount.tags,
			code: discount.code,
			product_ids: discount.product_ids,
			discount_kind,
			discount_amount,
			discount_currency,
			active_from: discount.active_period.start.timestamp_millis(),
			active_to: discount.active_period.end.timestamp_millis(),
			max_uses: discount.max_uses,
			remaining_uses: discount.remaining_uses,
			max_uses_per_user: discount.max_uses_per_user,
			created_by: discount.created_by,
			created_at: discount.id.timestamp().timestamp_millis(),
			updated_at: discount.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, TypesenseCollection)]
#[typesense(collection_name = "redeem_codes")]
#[serde(deny_unknown_fields)]
pub struct RedeemCode {
	pub id: RedeemCodeId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub code: String,
	pub remaining_uses: i32,
	pub active_from: i64,
	pub active_to: i64,
	pub effects: Vec<String>,
	pub created_by: UserId,
	pub special_event_id: Option<SpecialEventId>,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::product::codes::RedeemCode> for RedeemCode {
	fn from(value: database::product::codes::RedeemCode) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			code: value.code,
			remaining_uses: value.remaining_uses,
			active_from: value.active_period.start.timestamp_millis(),
			active_to: value.active_period.end.timestamp_millis(),
			effects: value.effects.into_iter().map(|x| x.to_string()).collect(),
			special_event_id: value.special_event_id,
			created_by: value.created_by,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, TypesenseCollection)]
#[typesense(collection_name = "special_events")]
#[serde(deny_unknown_fields)]
pub struct SpecialEvent {
	pub id: SpecialEventId,
	pub name: String,
	pub description: Option<String>,
	pub tags: Vec<String>,
	pub created_by: UserId,
	#[typesense(default_sort)]
	pub created_at: i64,
	pub updated_at: i64,
	pub search_updated_at: i64,
}

impl From<database::product::codes::SpecialEvent> for SpecialEvent {
	fn from(value: database::product::codes::SpecialEvent) -> Self {
		Self {
			id: value.id,
			name: value.name,
			description: value.description,
			tags: value.tags,
			created_by: value.created_by,
			created_at: value.id.timestamp().timestamp_millis(),
			updated_at: value.updated_at.timestamp_millis(),
			search_updated_at: Utc::now().timestamp_millis(),
		}
	}
}

pub(super) fn typesense_collections() -> impl IntoIterator<Item = TypesenseGenericCollection> {
	[
		TypesenseGenericCollection::new::<DiscountCode>(),
		TypesenseGenericCollection::new::<RedeemCode>(),
		TypesenseGenericCollection::new::<SpecialEvent>(),
	]
}
