import {
	type DispatchPayload,
	DispatchType,
	type DispatchWorkerMessage,
	type SubscribeWorkerMessage,
	type UnsubscribeWorkerMessage,
	WorkerMessageType,
} from "./eventApiWorkerTypes";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function log(...args: any[]) {
	console.log(`[${self.name}]`, ...args);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function debug(...args: any[]) {
	console.debug(`[${self.name}]`, ...args);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function warn(...args: any[]) {
	console.warn(`[${self.name}]`, ...args);
}

let eventApi:
	| {
			open_socket?: WebSocket;
			queue: string[];
			subscriptions: Map<string, Set<string>>;
	  }
	| undefined = undefined;

const ports: MessagePort[] = [];

const RECONNECT_BASE_DELAY_MS = 1000;
const RECONNECT_MAX_DELAY_MS = 30000;
let reconnectAttempts = 0;

onconnect = (event) => {
	debug("new worker port connected");
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

function fromMapKey(key: string): { type: DispatchType; id: string } {
	const [type, id] = key.split(":");
	return { type: type as DispatchType, id };
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

	log(`connecting to ${import.meta.env.PUBLIC_EVENT_API_V3}`);
	const socket = new WebSocket(import.meta.env.PUBLIC_EVENT_API_V3);
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

function cleanUpSubscriptions() {
	if (!eventApi) {
		return;
	}

	for (const [topic, handlers] of eventApi.subscriptions) {
		if (handlers.size === 0) {
			const { type, id } = fromMapKey(topic);

			eventApi.subscriptions.delete(topic);

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
}

function subscribe(type: DispatchType, id: string, handlerId: string) {
	if (!eventApi) {
		eventApi = init();
	}

	const handlers = eventApi.subscriptions.get(mapKey(type, id));

	if (handlers) {
		handlers.add(handlerId);
	} else {
		const set = new Set<string>();
		set.add(handlerId);

		eventApi.subscriptions.set(mapKey(type, id), set);

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

function unsubscribe(type: DispatchType, id: string, handlerId: string) {
	if (!eventApi) {
		return;
	}

	const handlers = eventApi.subscriptions.get(mapKey(type, id));
	if (!handlers) {
		return;
	}

	log(handlerId);

	if (!handlers.delete(handlerId)) {
		return;
	}

	setTimeout(cleanUpSubscriptions, 500);
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
	debug("ws connected");
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
		debug(`got hello from ${hello.d.instance.name}, session: ${hello.d.session_id}`);
		reconnectAttempts = 0;

		if (eventApi) {
			eventApi.open_socket = this;

			debug(`sending ${eventApi.queue.length} queued messages`);
			for (const message of eventApi.queue) {
				this.send(message);
			}
		}

		return;
	}

	// Heartbeat
	if (data.op === 2) {
		debug("heartbeat");
		return;
	}

	// Reconnect
	if (data.op === 4) {
		debug("reconnect requested");
		this.close();
		reconnectWithBackoff();
		return;
	}

	// Error
	if (data.op === 6) {
		warn("error", data);
		return;
	}
}

function reconnectWithBackoff() {
	const delay = Math.min(RECONNECT_BASE_DELAY_MS * Math.pow(2, reconnectAttempts), RECONNECT_MAX_DELAY_MS);
	reconnectAttempts++;
	warn(`reconnecting in ${delay}ms (attempt ${reconnectAttempts})`);
	setTimeout(() => {
		eventApi = init();
	}, delay);
}

function onClose(this: WebSocket, event: CloseEvent) {
	if (event.wasClean) {
		log(`ws connection closed cleanly`, event.code, event.reason);
		reconnectAttempts = 0;
	} else {
		warn(`ws connection closed`, event.code, event.reason);
		reconnectWithBackoff();
	}

	// Reset
	eventApi = undefined;
}

function onDispatch(payload: DispatchPayload) {
	debug("received dispatch", payload);

	const handlers = eventApi?.subscriptions.get(mapKey(payload.type, payload.body.id));
	if (handlers) {
		debug(`emitting on ${ports.length} worker ports for ${handlers.size} handlers`);
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
