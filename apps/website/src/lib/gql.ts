import { cacheExchange, Client, createClient, fetchExchange, mapExchange } from "@urql/svelte";
import { get } from "svelte/store";
import { authExchange } from "@urql/exchange-auth";
import { sessionToken } from "$/lib/auth";
import { PUBLIC_GQL_API_V4 } from "$env/static/public";
import { currentError, errorDialogMode } from "./error";

// var is used on purpose here to prevent a weird error when calling the function below
// "Uncaught (in promise) ReferenceError: can't access lexical declaration 'client' before initialization"

// non-reactive on purpose
// eslint-disable-next-line no-var
var client: Client | undefined;

export function gqlClient(): Client {
	if (client) return client;

	client = createClient({
		url: PUBLIC_GQL_API_V4,
		exchanges: [
			mapExchange({
				onError(error) {
					console.error(error);
					// Error is already handled by authExchange
					if (error.response?.status !== 401) {
						currentError.set(error.message);
						errorDialogMode.set("shown");
					}
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
