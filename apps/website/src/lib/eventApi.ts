import EventApiWorker from "$/workers/eventApiWorker?sharedworker";
import { DispatchType, WorkerMessageType, type DispatchPayload, type DispatchWorkerMessage, type SubscribeWorkerMessage, type UnsubscribeWorkerMessage } from "$/workers/eventApiWorkerTypes";

function worker() {
	if (!window.EVENT_API_CALLBACKS) {
		window.EVENT_API_CALLBACKS = new Map();
	}

	if (!window.EVENT_API_WORKER) {
		const worker = new EventApiWorker({ name: "7TV EventAPI Worker" });
		worker.port.onmessage = workerOnMessage;

		window.EVENT_API_WORKER = worker;
	}

	return window.EVENT_API_WORKER;
}

function workerOnMessage(event: MessageEvent) {
	const payload: DispatchWorkerMessage = event.data;

	for (const handlerId of payload.handlerIds) {
		const handler = window.EVENT_API_CALLBACKS.get(handlerId);

		if (handler) {
			handler(payload.payload);
		}
	}
}

export function subscribe(type: DispatchType, id: string, handler: (pl: DispatchPayload) => void, handlerId?: string) {
	const w = worker();

	// Generate a random handler ID to identify this specific handler
	if (!handlerId) {
		handlerId = Math.floor(Math.random() * 1000_000_000).toString(16);
	}

	window.EVENT_API_CALLBACKS.set(handlerId, handler);

	const msg: SubscribeWorkerMessage = { type: WorkerMessageType.Subscribe, dispatchType: type, id, handlerId };
	w.port.postMessage(msg);

	return () => unsubscribe(type, id, handlerId);
}

export function unsubscribe(type: DispatchType, id: string, handlerId: string) {
	const w = worker();
	const msg: UnsubscribeWorkerMessage = { type: WorkerMessageType.Unsubscribe, dispatchType: type, id, handlerId };
	w.port.postMessage(msg);
}
