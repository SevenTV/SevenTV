import { redirect } from "@sveltejs/kit";
import { user } from "$/lib/auth";
import { get } from "svelte/store";

//If the user is not signed in, keep the url so that when the login
//process finished they will be redirected back to the accounts page
export function load() {
	if (get(user) !== null) {
		redirect(302, "/settings");
	}
}
