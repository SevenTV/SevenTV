import EventApiWorker from "$/workers/eventApiWorker?sharedworker";
import { DispatchType, WorkerMessageType, type DispatchPayload, type DispatchWorkerMessage, type SubscribeWorkerMessage, type UnsubscribeWorkerMessage } from "$/workers/eventApiWorkerTypes";

function worker() {
	if (!window.EVENT_API_CALLBACKS) {
		window.EVENT_API_CALLBACKS = new Map();
	}

	const worker = new EventApiWorker({ name: "EventAPI Worker" });

	worker.port.onmessage = (event) => {
		const payload: DispatchWorkerMessage = event.data;

		for (const handlerId of payload.handlerIds) {
			const handler = window.EVENT_API_CALLBACKS.get(handlerId);

			if (handler) {
				handler(payload.payload);
			}
		}
	};

	return worker;
}

export function subscribe(type: DispatchType, id: string, handler: (pl: DispatchPayload) => void) {
	const w = worker();

	// Generate a random handler ID to identify this specific handler
	const handlerId = Math.floor(Math.random() * 1000_000_000).toString(16);

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
