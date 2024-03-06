import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { DialogMode } from "$/components/dialogs/dialog.svelte";

export const user = writable(false);

export const showMobileMenu = writable(false);

export const uploadDialogMode = writable<DialogMode>(DialogMode.Hidden);

export const signInDialogMode = writable<DialogMode>(DialogMode.Hidden);

export enum Theme {
	System = "system-theme",
	Light = "light-theme",
	Dark = "dark-theme",
}

export const theme = writable<Theme | null>(loadTheme());

function loadTheme() {
	const savedTheme = browser && window.localStorage.getItem("theme");
	if (savedTheme) {
		return JSON.parse(savedTheme) as Theme;
	}
	return null;
}

theme.subscribe((value) => {
	if (value) {
		window.localStorage.setItem("theme", JSON.stringify(value));
	}
	if (!browser) return;
	for (const key in Theme) {
		document.documentElement.classList.remove(Theme[key as keyof typeof Theme]);
	}
	if (value) {
		document.documentElement.classList.add(value);
	}
});

// Layout

export enum Layout {
	SmallGrid = "small-grid",
	BigGrid = "big-grid",
	List = "list",
}

function loadLayout(key: string) {
	const savedLayout = browser && window.localStorage.getItem(key);
	if (savedLayout) {
		return JSON.parse(savedLayout) as Layout;
	}
	return Layout.BigGrid;
}

function saveLayout(key: string, value: Layout | null) {
	if (value && browser) {
		window.localStorage.setItem("discoverFollowingLayout", JSON.stringify(value));
	}
}

// Emotes
export const emotesLayout = writable(loadLayout("emotesLayout"));
emotesLayout.subscribe((value) => saveLayout("emotesLayout", value));

// Discover / Following
export const discoverFollowingLayout = writable(loadLayout("discoverFollowingLayout"));
discoverFollowingLayout.subscribe((value) => saveLayout("discoverFollowingLayout", value));
