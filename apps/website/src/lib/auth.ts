import { graphql } from "$/gql";
import type { User } from "$/gql/graphql";
import { PUBLIC_REST_API_V4 } from "$env/static/public";
import { derived, type Readable, writable } from "svelte/store";
import { gqlClient } from "./gql";
import { browser } from "$app/environment";

const LOCALSTORAGE_KEY = "7tv-token";

// Stores should be considered loading when their value is `undefined`
// Null means the value is known to be empty

export const sessionToken = writable<string | null | undefined>(
	browser ? window.localStorage.getItem(LOCALSTORAGE_KEY) : undefined,
);
export const user: Readable<User | null | undefined> = derived(sessionToken, (value, set) => {
	if (!value) {
		if (value === null) {
			set(null);
		}
		return;
	}

	fetchMe().then((user) => set(user));
});

// Save session token to localstorage when changed
sessionToken.subscribe(async (token) => {
	if (token) {
		localStorage.setItem(LOCALSTORAGE_KEY, token);
	} else if (token === null) {
		// Only reset session token when set to null (not undefined)
		localStorage.removeItem(LOCALSTORAGE_KEY);
	}
});

export async function fetchMe(): Promise<User | null> {
	const res = await gqlClient()
		.query(
			graphql(`
				query Me {
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
								activePaint {
									id
									name
									data {
										layers {
											id
											ty {
												__typename
												... on PaintLayerTypeSingleColor {
													color {
														hex
													}
												}
												... on PaintLayerTypeLinearGradient {
													angle
													repeating
													stops {
														at
														color {
															hex
														}
													}
												}
												... on PaintLayerTypeRadialGradient {
													repeating
													stops {
														at
														color {
															hex
														}
													}
													shape
												}
												... on PaintLayerTypeImage {
													images {
														url
														mime
														size
														scale
														width
														height
														frameCount
													}
												}
											}
											opacity
										}
										shadows {
											color {
												hex
											}
											offsetX
											offsetY
											blur
										}
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
							editableEmoteSetIds
							permissions {
								user {
									manageAny
								}
								emote {
									manageAny
								}
							}
						}
					}
				}
			`),
			{},
		)
		.toPromise();

	if (res.error || !res.data || !res.data.users.me) {
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

	sessionToken.set(null);
}
