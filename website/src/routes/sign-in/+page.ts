import { DialogMode } from "$/components/dialogs/dialog.svelte";
import { signInDialogMode } from "$/lib/stores";
import { redirect } from "@sveltejs/kit";

export function load() {
	signInDialogMode.set(DialogMode.Shown);
	redirect(301, "/");
}
