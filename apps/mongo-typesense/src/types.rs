pub mod mongo {
	pub use shared::database::automod::*;
	pub use shared::database::badge::*;
	pub use shared::database::emote::*;
	pub use shared::database::emote_moderation_request::*;
	pub use shared::database::emote_set::*;
	pub use shared::database::entitlement::*;
	pub use shared::database::page::*;
	pub use shared::database::paint::*;
	pub use shared::database::product::codes::*;
	pub use shared::database::product::invoice::*;
	pub use shared::database::product::promotion::*;
	pub use shared::database::product::subscription::*;
	pub use shared::database::product::subscription_timeline::*;
	pub use shared::database::product::*;
	pub use shared::database::role::*;
	pub use shared::database::stored_event::*;
	pub use shared::database::ticket::*;
	pub use shared::database::user::ban::*;
	pub use shared::database::user::ban_template::*;
	pub use shared::database::user::editor::*;
	pub use shared::database::user::relation::*;
	pub use shared::database::user::*;
}

pub mod typesense {
	pub use shared::typesense::types::automod::*;
	pub use shared::typesense::types::badge::*;
	pub use shared::typesense::types::emote::*;
	pub use shared::typesense::types::emote_moderation_request::*;
	pub use shared::typesense::types::emote_set::*;
	pub use shared::typesense::types::entitlement::*;
	pub use shared::typesense::types::event::*;
	pub use shared::typesense::types::page::*;
	pub use shared::typesense::types::paint::*;
	pub use shared::typesense::types::product::codes::*;
	pub use shared::typesense::types::product::invoice::*;
	pub use shared::typesense::types::product::promotion::*;
	pub use shared::typesense::types::product::subscription::*;
	pub use shared::typesense::types::product::subscription_timeline::*;
	pub use shared::typesense::types::product::*;
	pub use shared::typesense::types::role::*;
	pub use shared::typesense::types::ticket::*;
	pub use shared::typesense::types::user::ban::*;
	pub use shared::typesense::types::user::ban_template::*;
	pub use shared::typesense::types::user::editor::*;
	pub use shared::typesense::types::user::relation::*;
	pub use shared::typesense::types::user::*;
}
