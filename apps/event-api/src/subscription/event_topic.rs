use std::hash::{Hash, Hasher};

use shared::database::Id;
use shared::event::EventUserPresencePlatform;
use shared::event_api::payload::{SubscribeCondition, SubscribeConditionChannelPlatform};
use shared::event_api::types::EventType;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A event topic is a combination of an event type and a set of conditions.
/// The conditions are hashed and using sha256 and the resulting hash is used.
/// Due to the large nature of this sturct (34 bytes) its not recommended to use
/// it as a key in a hashmap. You can use the `as_key` method to get a key that
/// is only 16 bytes large. This key is not guaranteed to be unique but almost
/// always is.
pub struct EventTopic {
	pub event: EventType,
	pub scope: EventScope,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventScope {
	Id(Id<()>),
	Presence(EventUserPresencePlatform),
}

impl TryFrom<SubscribeCondition> for EventScope {
	type Error = ();

	fn try_from(value: SubscribeCondition) -> Result<Self, Self::Error> {
		match value {
			SubscribeCondition::ObjectId { object_id } => Ok(Self::Id(object_id)),
			SubscribeCondition::Channel { id, platform: SubscribeConditionChannelPlatform::Twitch, .. } => Ok(Self::Presence(EventUserPresencePlatform::Twitch(id.parse().map_err(|_| ())?))),
			SubscribeCondition::Channel { id, platform: SubscribeConditionChannelPlatform::Kick, .. } => Ok(Self::Presence(EventUserPresencePlatform::Kick(id.parse().map_err(|_| ())?))),
			SubscribeCondition::Channel { id, platform: SubscribeConditionChannelPlatform::Youtube, .. } => Ok(Self::Presence(EventUserPresencePlatform::Youtube(id))),
			_ => Err(()),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TopicKey(pub EventType, pub u64);

impl EventTopic {
	/// Create a new event topic from a event type and a set of conditions.
	pub fn new(event: EventType, scope: EventScope) -> Self {
		Self { event, scope }
	}

	/// Copy the event conditions but with a different event type.
	pub fn copy_scope(&self, event: EventType) -> Self {
		Self {
			event,
			scope: self.scope.clone(),
		}
	}

	/// Convert the event topic into a key that is only 16 bytes large.
	pub fn as_key(&self) -> TopicKey {
		let mut hasher = fnv::FnvHasher::default();
		self.scope.hash(&mut hasher);
		TopicKey(self.event, hasher.finish())
	}
}
