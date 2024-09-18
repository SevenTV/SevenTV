import { writable } from "svelte/store";

export const sessionToken = writable<string | null>(undefined);
export const user = writable<boolean | null>(false);
