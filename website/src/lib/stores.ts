import { writable } from "svelte/store";

export const user = writable(false);

export const showMobileMenu = writable(false);

export const showUploadDialog = writable(false);
