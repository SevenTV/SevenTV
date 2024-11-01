import { browser } from "$app/environment";
import { writable } from "svelte/store";

const LOCAL_STORAGE_KEY = "7tv-token";

export const sessionToken = writable<string | null>(
	(browser && window.localStorage.getItem(LOCAL_STORAGE_KEY)) || null,
);
export const user = writable<boolean | null>(false);

if (browser) {
	sessionToken.subscribe((value) => {
		if (value) {
			window.localStorage.setItem(LOCAL_STORAGE_KEY, value);
		} else {
			window.localStorage.removeItem(LOCAL_STORAGE_KEY);
		}
	});
}
