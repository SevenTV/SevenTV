import { createGqlClient } from "$/lib/gql";
import { sessionToken, user } from "$/store/auth";
import { waitLocale } from "svelte-i18n";

export const ssr = false;

export async function load() {
	await waitLocale();

	sessionToken.set(window.localStorage.getItem("auth_sessionToken"));
	const client = createGqlClient();
	// Save session token to localstorage when changed
	sessionToken.subscribe((token) => {
		if (token) {
			localStorage.setItem("auth_sessionToken", token);
			// TODO: Request user
		} else if (token === null) {
			// Only reset session token when set to null (not undefined)
			localStorage.removeItem("auth_sessionToken");
			user.set(null);
		}
	});

	return {
		client,
	};
}
