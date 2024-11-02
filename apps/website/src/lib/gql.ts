import { createClient, fetchExchange, cacheExchange, type Exchange, Client } from "@urql/svelte";
import { get } from "svelte/store";
import { authExchange } from "@urql/exchange-auth";
import { sessionToken } from "$/store/auth";
import { PUBLIC_GQL_API_V4 } from "$env/static/public";

export function createGqlClient(): Client {
	const exchanges: Exchange[] = [cacheExchange];

	exchanges.push(
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
	);

	exchanges.push(fetchExchange);

	return createClient({
		url: PUBLIC_GQL_API_V4,
		exchanges,
	});
}
