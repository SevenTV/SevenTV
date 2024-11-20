// See https://kit.svelte.dev/docs/types#app

import type { DispatchPayload } from "./lib/eventApi";

// for information about these interfaces
declare global {
	namespace App {
		interface Error {
			message?: string | null;
			details?: string | null;
		}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
	interface Window {
		EVENT_API_CALLBACKS: Map<string, (pl: DispatchPayload) => void>;
	}
}

export { };
