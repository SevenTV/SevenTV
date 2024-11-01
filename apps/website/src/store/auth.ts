import { graphql } from "$/gql";
import type { User } from "$/gql/graphql";
import { Client } from "@urql/svelte";
import { writable } from "svelte/store";

// Stores should be considered loading when their value is `undefined`
// Null means the value is known to be empty

export const sessionToken = writable<string | null>(undefined);
export const user = writable<User | null>(undefined);

export async function fetchMe(client: Client) {
	const res = await client.query(graphql(`query Me {
		users {
			me {
				id
			}
		}
	}`), {}).toPromise();

	if (res.error || !res.data || !res.data.users.me) {
		if (res.error) {
			console.error(res.error);
		}

		user.set(null);
		return;
	}

	user.set(res.data.users.me as User);
}
