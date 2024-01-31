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
	const sideBar = window.localStorage.getItem("sideBar");
	if (!sideBar) {
		return true;
	}
	return sideBar === "true";
}

sideBar.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem("sideBar", value.toString());
	}
});
