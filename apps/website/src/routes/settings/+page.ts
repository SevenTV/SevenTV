import { redirect } from "@sveltejs/kit";
import { user } from "$/lib/auth";
import { get } from "svelte/store";

export function load() {
	if (!get(user)) {
		redirect(302, "/settings/appearance");
	}
}
