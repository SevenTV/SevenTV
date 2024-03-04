import { showSignInDialog } from "$/lib/stores";
import { redirect } from "@sveltejs/kit";

export function load() {
	showSignInDialog.set(true);
	redirect(301, "/");
}
