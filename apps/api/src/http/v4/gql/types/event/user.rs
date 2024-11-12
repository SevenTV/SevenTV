use shared::database::stored_event::StoredEventUserData;

#[derive(async_graphql::Union)]
pub enum EventUserData {
	Create(EventUserDataCreate),
	ChangeActivePaint(EventUserDataChangeActivePaint),
	ChangeActiveBadge(EventUserDataChangeActiveBadge),
	ChangeActiveEmoteSet(EventUserDataChangeActiveEmoteSet),
	AddConnection(EventUserDataAddConnection),
	RemoveConnection(EventUserDataRemoveConnection),
	AddEntitlement(EventUserDataAddEntitlement),
	RemoveEntitlement(EventUserDataRemoveEntitlement),
	Merge(EventUserDataMerge),
	Delete(EventUserDataDelete),
}

impl From<StoredEventUserData> for EventUserData {
	fn from(value: StoredEventUserData) -> Self {

	}
}
