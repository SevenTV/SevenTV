import type { DialogMode } from "$/components/dialogs/dialog.svelte";
import { writable } from "svelte/store";

export const currentError = writable<string | undefined>();
export const errorDialogMode = writable<DialogMode>("hidden");
