import { writable } from "svelte/store";
import { browser } from "$app/environment";

export const user = writable(false);

export const showMobileMenu = writable(false);

export const showUploadDialog = writable(false);

export enum Theme {
    System = "system",
    Light = "light",
    Dark = "dark",
};

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
    switch (value) {
        case null:
        case Theme.System:
            document.documentElement.classList.remove('light', 'dark');
            break;
        case Theme.Light:
            document.documentElement.classList.remove('dark');
            document.documentElement.classList.add('light');
            break;
        case Theme.Dark:
            document.documentElement.classList.remove('light');
            document.documentElement.classList.add('dark');
            break;
    }
});
