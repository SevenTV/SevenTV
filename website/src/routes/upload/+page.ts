import { user, showUploadDialog } from "$/lib/stores";
import { redirect } from "@sveltejs/kit";
import { get } from "svelte/store";

export function load() {
	if (!get(user)) {
		redirect(303, `/sign-in?r=${encodeURI("/upload")}`);
	}
	showUploadDialog.set(true);
	redirect(303, "/");
}
