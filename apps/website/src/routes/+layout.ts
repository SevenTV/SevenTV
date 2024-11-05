import { waitLocale } from "svelte-i18n";

export const ssr = false;

export async function load() {
	await waitLocale();
}
