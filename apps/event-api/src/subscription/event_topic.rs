use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use sha2::Digest;
use shared::event_api::types::EventType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A event topic is a combination of an event type and a set of conditions.
/// The conditions are hashed and using sha256 and the resulting hash is used.
/// Due to the large nature of this sturct (34 bytes) its not recommended to use
/// it as a key in a hashmap. You can use the `as_key` method to get a key that
/// is only 16 bytes large. This key is not guaranteed to be unique but almost
/// always is.
pub struct EventTopic {
	pub event: EventType,
	pub cond_hash: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TopicKey(pub EventType, pub u64);

impl EventTopic {
	/// Create a new event topic from a event type and a set of conditions.
	pub fn new<K: AsRef<[u8]> + std::cmp::Ord, V: AsRef<[u8]>>(event: EventType, conds: &HashMap<K, V>) -> Self {
		let cond_hash = if conds.is_empty() {
			None
		} else {
			let mut conds = conds.iter().collect::<Vec<_>>();

			conds.sort_by(|a, b| a.0.cmp(b.0));

			let mut hash = sha2::Sha256::new();

			for (key, value) in conds {
				hash.update(key.as_ref());
				hash.update(value.as_ref());
			}

			Some(hash.finalize().into())
		};

		Self { event, cond_hash }
	}

	/// Copy the event conditions but with a different event type.
	pub fn copy_cond(&self, event: EventType) -> Self {
		Self { event, cond_hash: self.cond_hash }
	}

	/// Convert the event topic into a key that is only 16 bytes large.
	pub fn as_key(&self) -> TopicKey {
		let mut hasher = fnv::FnvHasher::default();
		self.hash(&mut hasher);
		TopicKey(self.event, hasher.finish())
	}
}

impl std::fmt::Display for EventTopic {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.cond_hash {
			Some(cond_hash) => write!(f, "events.op.dispatch.type.{}.{}", self.event, hex::encode(cond_hash)),
			None => write!(f, "events.op.dispatch.type.{}", self.event),
		}
	}
}

impl FromStr for EventTopic {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.strip_prefix("events.op.dispatch.type.").ok_or(())?;

		let event = s.split('.').collect::<Vec<_>>();

		let (event, cond) = if event.len() <= 2 {
			(event.join("."), None)
		} else if event.len() > 2 {
			(event[..event.len() - 1].join("."), Some(event[event.len() - 1]))
		} else {
			return Err(());
		};

		let event = event.parse().map_err(|_| ())?;

		let mut cond_hash = [0u8; 32];

		let cond_hash = cond
			.map(|hex| hex::decode_to_slice(hex, &mut cond_hash).map(|_| cond_hash))
			.transpose()
			.map_err(|_| ())?;

		Ok(Self { event, cond_hash })
	}
}
