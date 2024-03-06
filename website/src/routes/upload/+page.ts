import { DialogMode } from "$/components/dialogs/dialog.svelte";
import { user, uploadDialogMode, signInDialogMode } from "$/lib/stores";
import { redirect } from "@sveltejs/kit";
import { get } from "svelte/store";

export function load() {
	if (!get(user)) {
		signInDialogMode.set(DialogMode.Shown);
	} else {
		uploadDialogMode.set(DialogMode.Shown);
	}
	redirect(303, "/");
}
