import { writable } from "svelte/store";
import { browser } from "$app/environment";
import { type DialogMode } from "$/components/dialogs/dialog.svelte";

export const showMobileMenu = writable(false);

export const uploadDialogMode = writable<DialogMode>("hidden");

export const signInDialogMode = writable<DialogMode>("hidden");

export const defaultEmoteSetDialogMode = writable<DialogMode>("hidden");

export type Theme = "system-theme" | "light-theme" | "dark-theme";

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

  document.documentElement.classList.remove(
    "system-theme",
    "light-theme",
    "dark-theme",
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
export const discoverFollowingLayout = writable(
  loadLayout("discoverFollowingLayout"),
);
discoverFollowingLayout.subscribe((value) =>
  saveLayout("discoverFollowingLayout", value)
);

// Admin tickets

export const adminTicketsLayout = writable(
  loadLayout("adminTicketsLayout", "list"),
);
adminTicketsLayout.subscribe((value) =>
  saveLayout("adminTicketsLayout", value)
);
