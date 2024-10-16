import { DialogMode } from "$/components/dialogs/dialog.svelte";
import { uploadDialogMode, signInDialogMode } from "$/store/layout";
import { user } from "$/store/auth";
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
