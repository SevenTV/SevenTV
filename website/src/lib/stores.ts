import { browser } from "$app/environment";
import { writable } from "svelte/store";

export const user = writable(false);

export const showMobileMenu = writable(false);

export const sideBar = writable(sideBarInit());

function sideBarInit() {
	// Show sidebar by default
	if (!browser) {
		return true;
	}
	return window.localStorage.getItem("sideBar") === "true";
}

sideBar.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem("sideBar", value.toString());
	}
});
