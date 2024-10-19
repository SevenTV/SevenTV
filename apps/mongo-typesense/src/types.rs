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
	pub use shared::database::product::special_event::*;
	pub use shared::database::product::subscription::*;
	pub use shared::database::product::*;
	pub use shared::database::role::*;
	pub use shared::database::stored_event::*;
	pub use shared::database::ticket::*;
	pub use shared::database::user::ban::*;
	pub use shared::database::user::ban_template::*;
	pub use shared::database::user::editor::*;
	pub use shared::database::user::profile_picture::*;
	pub use shared::database::user::relation::*;
	pub use shared::database::user::*;
}

pub mod typesense {
	pub use shared::typesense::types::product::special_event::*;
	pub use shared::typesense::types::product::subscription::*;
	pub use shared::typesense::types::product::*;
	pub use shared::typesense::types::role::*;
	pub use shared::typesense::types::user::*;
}
