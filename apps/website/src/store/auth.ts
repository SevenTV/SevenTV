import { graphql } from "$/gql";
import type { User } from "$/gql/graphql";
import { PUBLIC_REST_API_V4 } from "$env/static/public";
import { Client, getContextClient } from "@urql/svelte";
import { derived, writable, type Readable } from "svelte/store";

// Stores should be considered loading when their value is `undefined`
// Null means the value is known to be empty

const client = getContextClient();

export const sessionToken = writable<string | null>(undefined);
export const user: Readable<User | null | undefined> = derived(sessionToken, (value, set) => {
	console.log("fetching user", value);

	if (value === undefined) {
		return;
	}

	fetchMe(client).then((user) => set(user));
});

export async function fetchMe(client: Client): Promise<User | null> {
	const res = await client.query(graphql(`query Me {
		users {
			me {
				id
				mainConnection {
					platformDisplayName
					platformAvatarUrl
				}
				style {
					activeProfilePicture {
						images {
							url
							mime
							size
							width
							height
							scale
							frameCount
						}
					}
				}
				highestRoleColor {
					hex
				}
				roles {
					name
					color {
						hex
					}
				}
			}
		}
	}`), {}).toPromise();

	if (res.error || !res.data || !res.data.users.me) {
		if (res.error) {
			console.error(res.error);
		}

		return null;
	}

	return res.data.users.me as User;
}

export async function logout() {
	const res = await fetch(`${PUBLIC_REST_API_V4}/auth/logout`, {
		method: "POST",
		credentials: "include",
	});

	if (!res.ok) {
		console.error(await res.json());
		return;
	}

	console.log("logged out");

	sessionToken.set(null);
}
