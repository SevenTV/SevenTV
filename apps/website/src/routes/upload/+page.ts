import { signInDialogMode, uploadDialogMode } from "$/lib/layout";
import { user } from "$/lib/auth";
import { redirect } from "@sveltejs/kit";
import { get } from "svelte/store";

export function load() {
    if (!get(user)) {
        signInDialogMode.set("shown");
    } else {
        uploadDialogMode.set("shown");
    }
    redirect(303, "/");
}
