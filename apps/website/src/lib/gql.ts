import { cacheExchange, Client, createClient, type Exchange, fetchExchange, mapExchange } from "@urql/svelte";
import { get } from "svelte/store";
import { authExchange } from "@urql/exchange-auth";
import { sessionToken } from "$/lib/auth";
import { PUBLIC_GQL_API_V4 } from "$env/static/public";
import { currentError, errorDialogMode } from "./error";

// non-reactive on purpose
let client: Client | undefined = undefined;

export function gqlClient(): Client {
	if (client) return client;

	client = createClient({
		url: PUBLIC_GQL_API_V4,
		exchanges: [
			mapExchange({
				onError(error) {
					console.error(error);
					currentError.set(error.message);
					errorDialogMode.set("shown");
				},
			}),
			cacheExchange,
			authExchange(async (utils) => {
				return {
					addAuthToOperation(operation) {
						const token = get(sessionToken);
						if (!token) return operation;
						return utils.appendHeaders(operation, {
							Authorization: `Bearer ${token}`,
						});
					},
					didAuthError(error) {
						return error.response?.status === 401;
					},
					async refreshAuth() {
						sessionToken.set(null);
					},
				};
			}),
			fetchExchange,
		],
	});

	return client;
}
