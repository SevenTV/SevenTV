import { graphql } from "$/gql";
import { UserEditorState, type User } from "$/gql/graphql";
import { PUBLIC_REST_API_V4, PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
import { derived, get, writable } from "svelte/store";
import { gqlClient } from "./gql";
import { browser } from "$app/environment";
import { defaultEmoteSet } from "./defaultEmoteSet";

const LOCALSTORAGE_KEY = "7tv-token";

// Stores should be considered loading when their value is `undefined`
// Null means the value is known to be empty

export const sessionToken = writable<string | null | undefined>(
	browser ? window.localStorage.getItem(LOCALSTORAGE_KEY) : undefined,
);
export const user = writable<User | null | undefined>(undefined);
export const isSubscribed = derived(user, ($user) => $user?.billing.subscriptionInfo.activePeriod)

export function refreshUser() {
	fetchMe().then((data) => user.set(data));
}

sessionToken.subscribe(async (value) => {
	if (!value) {
		if (value === null) {
			user.set(null);
		}
		return;
	}

	refreshUser();
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

export const pendingEditorFor = writable(0);

user.subscribe((value) => {
	if (!value) {
		return 0;
	}

	pendingEditorFor.set(value.editorFor.filter((e) => e.state === UserEditorState.Pending).length);
});

export async function fetchMe(): Promise<User | null> {
	const res = await gqlClient()
		.query(
			graphql(`
				query Me($productId: Id!) {
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
								activeEmoteSetId
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
								admin {
									manageRedeemCodes
									manageEntitlements
								}
								user {
									manageAny
									useCustomProfilePicture
									manageBilling
									manageSessions
								}
								emote {
									manageAny
								}
								emoteSet {
									manage
									manageAny
								}
								ticket {
									create
								}
							}
							billing(productId: $productId) {
								subscriptionInfo {
									activePeriod {
										providerId {
											provider
										}
									}
								}
							}
							inventory {
								products {
									to {
										productId
									}
								}
							}
							editorFor {
								user {
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
										activeEmoteSetId
									}
									highestRoleColor {
										hex
									}
								}
								state
							}
						}
					}
				}
			`),
			{
				productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
			},
		)
		.toPromise();

	if (res.error || !res.data || !res.data.users.me) {
		return null;
	}

	return res.data.users.me as User;
}

export async function logout() {
	const token = get(sessionToken);

	if (!token) {
		return;
	}

	const res = await fetch(`${PUBLIC_REST_API_V4}/auth/logout`, {
		method: "POST",
		credentials: "include",
		headers: {
			Authorization: `Bearer ${token}`,
		},
	});

	if (!res.ok) {
		console.error(await res.json());
		return;
	}

	sessionToken.set(null);
	defaultEmoteSet.set(undefined);
}
