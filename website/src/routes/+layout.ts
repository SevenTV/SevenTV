import { waitLocale } from "svelte-i18n";

export async function load() {
    await waitLocale();
}
