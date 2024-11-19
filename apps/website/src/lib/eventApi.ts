import { browser } from "$app/environment";
import { PUBLIC_EVENT_API_V3 } from "$env/static/public";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function log(...args: any[]) {
	console.log("[EventAPI]", ...args);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function warn(...args: any[]) {
	console.warn("[EventAPI]", ...args);
}

function mapKey(type: DispatchType, id: string) {
	return `${type}:${id}`;
}

function init() {
	if (window.EVEMT_API?.open_socket) {
		log("closing existing connection");
		window.EVEMT_API.open_socket.close();
		window.EVEMT_API = undefined;
	}

	log(`connecting to ${PUBLIC_EVENT_API_V3}`);
	const socket = new WebSocket(PUBLIC_EVENT_API_V3);
	socket.onmessage = onMessage;
	socket.onclose = onClose;
	socket.onopen = onOpen;

	return { queue: [], subscriptions: new Map() };
}

function socketSend(payload: string) {
	if (!browser) {
		return;
	}

	if (!window.EVEMT_API) {
		warn("not connected");
		return;
	}

	if (window.EVEMT_API.open_socket) {
		window.EVEMT_API.open_socket.send(payload);
	} else {
		log("socket not open, queueing");
		window.EVEMT_API.queue.push(payload);
	}
}

export function subscribe(type: DispatchType, id: string, handler: (pl: DispatchPayload) => void) {
	if (!browser) {
		return;
	}

	if (!window.EVEMT_API) {
		window.EVEMT_API = init();
	}

	const handlers = window.EVEMT_API.subscriptions.get(mapKey(type, id));
	if (handlers) {
		handlers.push(handler);
	} else {
		window.EVEMT_API.subscriptions.set(mapKey(type, id), [handler]);

		const payload: SubscribeMessage = {
			op: 35,
			d: {
				type,
				condition: {
					object_id: id,
				},
			}
		};

		log("subscribing to", type, id);
		socketSend(JSON.stringify(payload));
	}

	return () => unsubscribe(type, id, handler);
}

export function unsubscribe(type: DispatchType, id: string, handler: (pl: DispatchPayload) => void) {
	if (!browser) {
		return;
	}

	if (!window.EVEMT_API) {
		window.EVEMT_API = init();
	}

	const handlers = window.EVEMT_API.subscriptions.get(mapKey(type, id));
	if (!handlers) {
		return;
	}

	const index = handlers.indexOf(handler);

	if (index === -1) {
		return;
	}

	handlers.splice(index, 1);

	if (handlers.length === 0) {
		window.EVEMT_API.subscriptions.delete(mapKey(type, id));

		const payload: UnsubscribeMessage = {
			op: 36,
			d: {
				type,
				condition: {
					object_id: id,
				},
			}
		};

		log("unsubscribing from", type, id);
		socketSend(JSON.stringify(payload));
	}
}

interface HelloMessage {
	op: 1;
	d: {
		instance: {
			name: string;
		};
		session_id: string;
	};
}

interface DispatchMessage {
	op: 0;
	d: DispatchPayload;
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
	old_value?: object;
	value?: object | ChangeField[];
}

interface SubscribeMessage {
	op: 35;
	d: {
		type: DispatchType;
		condition?: { [key: string]: string };
	};
}

interface UnsubscribeMessage {
	op: 36;
	d: {
		type: DispatchType;
		condition?: { [key: string]: string };
	};
}

function onOpen(this: WebSocket) {
	log("connected");
}

function onMessage(this: WebSocket, event: MessageEvent) {
	const data = JSON.parse(event.data);

	// Dispatch
	if (data.op === 0) {
		const dispatch = data as DispatchMessage;
		onDispatch(dispatch.d);
		return;
	}

	// Hello
	if (data.op === 1) {
		const hello = data as HelloMessage;
		log(`got hello from ${hello.d.instance.name}, session: ${hello.d.session_id}`);

		if (window.EVEMT_API) {
			window.EVEMT_API.open_socket = this;

			log(`sending ${window.EVEMT_API.queue.length} queued messages`);
			for (const message of window.EVEMT_API.queue) {
				this.send(message);
			}
		}

		return;
	}

	// Heartbeat
	if (data.op === 2) {
		log("heartbeat");
		return;
	}

	// Reconnect
	if (data.op === 4) {
		log("reconnect requested");
		this.close();

		// Retry after 1 second
		setTimeout(() => {
			window.EVEMT_API = init();
		}, 1000);

		return;
	}

	// Error
	if (data.op === 6) {
		warn("error", data);
		return;
	}
}

function onClose(this: WebSocket, event: CloseEvent) {
	if (event.wasClean) {
		log(`connection closed cleanly`, event.code, event.reason);
	} else {
		warn(`connection closed`, event.code, event.reason);

		// Retry after 1 second
		setTimeout(() => {
			window.EVEMT_API = init();
		}, 1000);
	}

	// Reset
	window.EVEMT_API = undefined;
}

function onDispatch(payload: DispatchPayload) {
	log("dispatch", payload);

	const handlers = window.EVEMT_API?.subscriptions.get(mapKey(payload.type, payload.body.id));
	if (handlers) {
		handlers.forEach(handler => handler(payload));
	}
}
