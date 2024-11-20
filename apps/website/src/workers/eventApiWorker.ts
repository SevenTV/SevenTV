// import { PUBLIC_EVENT_API_V3 } from "$env/static/public";
import { type DispatchPayload, DispatchType, type DispatchWorkerMessage, type SubscribeWorkerMessage, type UnsubscribeWorkerMessage, WorkerMessageType } from "./eventApiWorkerTypes";

const PUBLIC_EVENT_API_V3 = "wss://events.7tv.io/v3";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function log(...args: any[]) {
	console.log("[EventAPI Worker]", ...args);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function warn(...args: any[]) {
	console.warn("[EventAPI Worker]", ...args);
}

let eventApi: {
	open_socket?: WebSocket;
	queue: string[];
	subscriptions: Map<string, string[]>;
} | undefined = undefined;

let ports: MessagePort[] = [];

onconnect = (event) => {
	const port = event.ports[0];
	ports.push(port);

	port.onmessage = (event) => {
		const type: WorkerMessageType = event.data.type;

		if (type === WorkerMessageType.Subscribe) {
			const message = event.data as SubscribeWorkerMessage;
			subscribe(message.dispatchType, message.id, message.handlerId);
		} else if (type === WorkerMessageType.Unsubscribe) {
			const message = event.data as UnsubscribeWorkerMessage;
			unsubscribe(message.dispatchType, message.id, message.handlerId);
		}
	};
};

function mapKey(type: DispatchType, id: string) {
	return `${type}:${id}`;
}

function reset() {
	if (eventApi?.open_socket) {
		log("closing existing connection");
		eventApi.open_socket.close();
		eventApi = undefined;
	}
}

function init() {
	reset();

	log(`connecting to ${PUBLIC_EVENT_API_V3}`);
	const socket = new WebSocket(PUBLIC_EVENT_API_V3);
	socket.onmessage = onMessage;
	socket.onclose = onClose;
	socket.onopen = onOpen;

	return { queue: [], subscriptions: new Map() };
}

function socketSend(payload: string) {
	if (!eventApi) {
		warn("not connected");
		return;
	}

	if (eventApi.open_socket) {
		eventApi.open_socket.send(payload);
	} else {
		eventApi.queue.push(payload);
	}
}

function subscribe(type: DispatchType, id: string, handlerId: string) {
	if (!eventApi) {
		eventApi = init();
	}

	const handlers = eventApi.subscriptions.get(mapKey(type, id));

	if (handlers) {
		handlers.push(handlerId);
	} else {
		eventApi.subscriptions.set(mapKey(type, id), [handlerId]);

		const payload: SubscribeMessage = {
			op: 35,
			d: {
				type,
				condition: {
					object_id: id,
				},
			},
		};

		log("subscribing to", type, id);
		socketSend(JSON.stringify(payload));
	}
}

function unsubscribe(
	type: DispatchType,
	id: string,
	handlerId: string,
) {
	if (!eventApi) {
		eventApi = init();
	}

	const handlers = eventApi.subscriptions.get(mapKey(type, id));
	if (!handlers) {
		return;
	}

	const index = handlers.indexOf(handlerId);

	if (index === -1) {
		return;
	}

	handlers.splice(index, 1);

	if (handlers.length === 0) {
		eventApi.subscriptions.delete(mapKey(type, id));

		const payload: UnsubscribeMessage = {
			op: 36,
			d: {
				type,
				condition: {
					object_id: id,
				},
			},
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

		if (eventApi) {
			eventApi.open_socket = this;

			log(`sending ${eventApi.queue.length} queued messages`);
			for (const message of eventApi.queue) {
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
			eventApi = init();
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
			eventApi = init();
		}, 1000);
	}

	// Reset
	eventApi = undefined;
}

function onDispatch(payload: DispatchPayload) {
	log("dispatch", payload);

	const handlers = eventApi?.subscriptions.get(mapKey(payload.type, payload.body.id));
	if (handlers) {
		for (const port of ports) {
			handlers.forEach((handler) => {
				port.postMessage({
					handlerIds: [handler],
					payload,
				} as DispatchWorkerMessage);
			});
		}
	}
}
