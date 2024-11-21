export interface DispatchWorkerMessage {
	handlerIds: string[];
	payload: DispatchPayload;
}

export enum WorkerMessageType {
	Subscribe,
	Unsubscribe,
}

export interface SubscribeWorkerMessage {
	type: WorkerMessageType.Subscribe;
	dispatchType: DispatchType;
	id: string;
	handlerId: string;
}

export interface UnsubscribeWorkerMessage {
	type: WorkerMessageType.Unsubscribe;
	dispatchType: DispatchType;
	id: string;
	handlerId: string;
}

export interface DispatchPayload {
	type: DispatchType;
	body: {
		id: string;
		kind: number;
		added?: ChangeField[];
		updated?: ChangeField[];
		removed?: ChangeField[];
		pushed?: ChangeField[];
		pulled?: ChangeField[];
	};
}

// https://github.com/seventv/eventapi?tab=readme-ov-file#subscription-types-1
export enum DispatchType {
	SystemAnnouncement = "system.announcement",
	EmoteCreate = "emote.create",
	EmoteUpdate = "emote.update",
	EmoteDelete = "emote.delete",
	// EmoteAll = "emote.*",
	EmoteSetCreate = "emote_set.create",
	EmoteSetUpdate = "emote_set.update",
	EmoteSetDelete = "emote_set.delete",
	// EmoteSetAll = "emote_set.*",
	UserCreate = "user.create",
	UserUpdate = "user.update",
	UserDelete = "user.delete",
	UserAddConnection = "user.add_connection",
	UserUpdateConnection = "user.update_connection",
	UserDeleteConnection = "user.delete_connection",
	// UserAll = "user.*",
	CosmeticCreate = "cosmetic.create",
	CosmeticUpdate = "cosmetic.update",
	CosmeticDelete = "cosmetic.delete",
	// CosmeticAll = "cosmetic.*",
	EntitlementCreate = "entitlement.create",
	EntitlementUpdate = "entitlement.update",
	EntitlementDelete = "entitlement.delete",
	// EntitlementAll = "entitlement.*",
}

export interface ChangeField {
	key: string;
	index?: number;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	old_value?: any;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	value?: any;
}
