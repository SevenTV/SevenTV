import { createGqlClient } from "$/lib/gql";
import { fetchMe, sessionToken, user } from "$/store/auth";
import { waitLocale } from "svelte-i18n";

export const ssr = false;

const LOCALSTORAGE_KEY = "7tv-token";

export async function load() {
	await waitLocale();

	const client = createGqlClient();
	// Save session token to localstorage when changed
	sessionToken.subscribe(async (token) => {
		if (token) {
			localStorage.setItem(LOCALSTORAGE_KEY, token);
			await fetchMe(client);
		} else if (token === null) {
			// Only reset session token when set to null (not undefined)
			localStorage.removeItem(LOCALSTORAGE_KEY);
			user.set(null);
		}
	});

	sessionToken.set(window.localStorage.getItem(LOCALSTORAGE_KEY));

	return {
		client,
	};
}
