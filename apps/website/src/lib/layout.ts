import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { type DialogMode } from "$/components/dialogs/dialog.svelte";

export const showMobileMenu = writable(false);

export const showConstructionBar = writable(loadConstructionBarShown());

export const uploadDialogMode = writable<DialogMode>("hidden");

export const signInDialogMode = writable<DialogMode>("hidden");
export const signInDialogPayload = writable<object | undefined>();

export const defaultEmoteSetDialogMode = writable<DialogMode>("hidden");

export type Theme = "system-theme" | "light-theme" | "dark-theme";

export const theme = writable<Theme>(loadTheme());

function loadConstructionBarShown() {
	if (!browser) return undefined;

	const savedValue = window.localStorage.getItem("showConstructionBar");
	if (savedValue) {
		return JSON.parse(savedValue) as boolean;
	}

	return true;
}

showConstructionBar.subscribe((value) => {
	if (browser) {
		window.localStorage.setItem("showConstructionBar", JSON.stringify(value));
	}
});

function loadTheme() {
	const savedTheme = browser && window.localStorage.getItem("theme");
	if (savedTheme) {
		return JSON.parse(savedTheme) as Theme;
	}
	return "dark-theme";
}

theme.subscribe((value) => {
	if (browser && value) {
		window.localStorage.setItem("theme", JSON.stringify(value));
	}
	if (!browser) return;

	document.documentElement.classList.remove("system-theme", "light-theme", "dark-theme");

	if (value) {
		document.documentElement.classList.add(value);
	}
});

export type ReducedMotion =
	| "reduced-motion-system"
	| "reduced-motion-enabled"
	| "reduced-motion-disabled";

export const reducedMotion = writable<ReducedMotion>(loadReducedMotion());

function loadReducedMotion() {
	const savedReducedMotion = browser && window.localStorage.getItem("reducedMotion");
	if (savedReducedMotion) {
		return JSON.parse(savedReducedMotion) as ReducedMotion;
	}
	return "reduced-motion-system";
}

reducedMotion.subscribe((value) => {
	if (browser && value) {
		window.localStorage.setItem("reducedMotion", JSON.stringify(value));
	}
	if (!browser) return;

	document.documentElement.classList.remove(
		"reduced-motion-system",
		"reduced-motion-enabled",
		"reduced-motion-disabled",
	);

	if (value) {
		document.documentElement.classList.add(value);
	}
});

// Layout

export type Layout = "small-grid" | "big-grid" | "list";

function loadLayout(key: string, defaultLayout?: Layout) {
	const savedLayout = browser && window.localStorage.getItem(key);
	if (savedLayout) {
		return JSON.parse(savedLayout) as Layout;
	}
	return defaultLayout ?? "big-grid";
}

function saveLayout(key: string, value: Layout | null) {
	if (value && browser) {
		window.localStorage.setItem(key, JSON.stringify(value));
	}
}

// Emotes
export const emotesLayout = writable(loadLayout("emotesLayout"));
emotesLayout.subscribe((value) => saveLayout("emotesLayout", value));

// Discover / Following
export const discoverFollowingLayout = writable(loadLayout("discoverFollowingLayout"));
discoverFollowingLayout.subscribe((value) => saveLayout("discoverFollowingLayout", value));

// Admin tickets
export const adminTicketsLayout = writable(loadLayout("adminTicketsLayout", "list"));
adminTicketsLayout.subscribe((value) => saveLayout("adminTicketsLayout", value));
