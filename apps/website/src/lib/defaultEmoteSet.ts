import { browser } from "$app/environment";
import { writable } from "svelte/store";

function getInitialValue() {
	const value = browser && window.localStorage.getItem("defaultEmoteSet");
	if (value) {
		return JSON.parse(value);
	}
	return undefined;
}

export const defaultEmoteSet = writable<string | undefined>(getInitialValue());

defaultEmoteSet.subscribe((value) => {
	if (browser && value) {
		window.localStorage.setItem("defaultEmoteSet", JSON.stringify(value));
	}
});
