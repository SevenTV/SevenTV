import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { type DialogMode } from "$/components/dialogs/dialog.svelte";
import { isEasterEvent, isSummerGiftEvent, isXmasEvent, isValentinesEvent } from "./events";

export const showMobileMenu = writable(false);

export const showConstructionBar = loadBarShown("showConstructionBar");
export const showEasterBar = loadBarShown("showEasterBar", !isEasterEvent());
export const showSummerGift = loadBarShown("showSummerGift", !isSummerGiftEvent());
export const showXmasBar = loadBarShown("showXmasBar", !isXmasEvent());
export const showValentinesBar = loadBarShown("showValentinesBar2026", !isValentinesEvent());

export const uploadDialogMode = writable<DialogMode>("hidden");
export const selectionModeInEmotes = writable<boolean>(false);

export const signInDialogMode = writable<DialogMode>("hidden");
export const signInDialogPayload = writable<object | undefined>();

export const defaultEmoteSetDialogMode = writable<DialogMode>("hidden");

export type Theme = "system-theme" | "light-theme" | "dark-theme";

export const theme = writable<Theme>(loadTheme());

function loadBarShown(key: string, disable?: boolean) {
	if (!browser) return writable(undefined);
	if (disable === true) return writable(false);

	let init = true;

	const savedValue = window.localStorage.getItem(key);
	if (savedValue) {
		init = JSON.parse(savedValue) as boolean;
	}

	const store = writable(init);
	store.subscribe((value) => {
		if (browser) {
			window.localStorage.setItem(key, JSON.stringify(value));
		}
	});

	return store;
}

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

export type PageRightLeftLayout = "ltr" | "rtl";

export const pageRightToLeft = writable<PageRightLeftLayout>(loadRightToLeft());

function loadRightToLeft(): PageRightLeftLayout {
	const savedRTL = browser && window.localStorage.getItem("pageRightToLeft");
	if (savedRTL) return savedRTL as PageRightLeftLayout;

	return "ltr";
}

pageRightToLeft.subscribe((isRtl) => {
	if (browser && isRtl) {
		if (!window.localStorage.getItem("pageRightToLeft")) {
			window.localStorage.setItem("pageRightToLeft", "ltr");
		}
		if (isRtl == "rtl") {
			document.documentElement.dir = "rtl";
			window.localStorage.setItem("pageRightToLeft", "rtl");
		} else {
			document.documentElement.dir = "ltr";
			window.localStorage.setItem("pageRightToLeft", "ltr");
		}
	}
});

// Time Format

export type FormatTime = "12h" | "24h";

export const timeFormat = writable<FormatTime>(loadTimeFormat());

function loadTimeFormat(): FormatTime {
	const savedTimeFormat = browser && window.localStorage.getItem("timeFormat");
	if (savedTimeFormat) return savedTimeFormat as FormatTime;

	const is24h = !new Intl.DateTimeFormat(navigator.language, { hour: 'numeric' })
		.format(0)
		.match(/AM|PM/);

	if (!is24h) {
		return "12h";
	} else {
		return "24h";
	}
}

timeFormat.subscribe((value) => {
	if (browser && value) window.localStorage.setItem("timeFormat", value);
});

// Date Format

export type FormatDate = "date-format-exact" | "date-format-relative";

export const dateFormat = writable<FormatDate>(loadFormatDate());

function loadFormatDate(): FormatDate {
	const savedDateFormat = browser && window.localStorage.getItem("dateFormat");
	if (savedDateFormat) return savedDateFormat as FormatDate;

	return "date-format-relative";
}

dateFormat.subscribe((value) => {
	if (browser && value) window.localStorage.setItem("dateFormat", value);
});

// Layout

export type Layout = "small-grid" | "big-grid" | "list" | "gallery";

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
