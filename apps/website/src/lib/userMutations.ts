import { graphql } from "$/gql";
import type { Platform, User } from "$/gql/graphql";
import { get } from "svelte/store";
import { gqlClient } from "./gql";
import { sessionToken } from "./auth";
import { PUBLIC_REST_API_V4 } from "$env/static/public";
import { currentError, errorDialogMode } from "./error";

export async function setActiveSet(userId: string, setId?: string) {
	const res = await gqlClient().mutation(
		graphql(`
			mutation SetActiveSet($userId: Id!, $setId: Id) {
				users {
					user(id: $userId) {
						activeEmoteSet(emoteSetId: $setId) {
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
								user {
									manageAny
									useCustomProfilePicture
								}
								emote {
									manageAny
								}
								emoteSet {
									manageAny
								}
								ticket {
									create
								}
							}
						}
					}
				}
			}
		`),
		{ userId, setId },
	);

	if (!res.data) {
		return undefined;
	}

	return res.data.users.user.activeEmoteSet as User;
}

export async function setActiveBadge(userId: string, badgeId?: string | null) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation SetActiveBadge($id: Id!, $badgeId: Id) {
					users {
						user(id: $id) {
							activeBadge(badgeId: $badgeId) {
								id
								connections {
									platform
									platformUsername
									platformDisplayName
								}
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
									activeBadgeId
									activeBadge {
										id
										name
										description
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
									activePaintId
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
								editors {
									editor {
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
									}
									state
								}
								editableEmoteSetIds
								permissions {
									user {
										manageAny
										useCustomProfilePicture
									}
									emote {
										manageAny
									}
									emoteSet {
										manageAny
									}
									ticket {
										create
									}
								}
							}
						}
					}
				}
			`),
			{
				id: userId,
				badgeId,
			},
		)
		.toPromise();

	if (!res.data) {
		throw res.error?.message;
	}

	return res.data.users.user.activeBadge as User;
}

export async function setActivePaint(userId: string, paintId?: string | null) {
	const res = await gqlClient()
		.mutation(
			graphql(`
				mutation SetActivePaint($id: Id!, $paintId: Id) {
					users {
						user(id: $id) {
							activePaint(paintId: $paintId) {
								id
								connections {
									platform
									platformUsername
									platformDisplayName
								}
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
									activeBadgeId
									activeBadge {
										id
										name
										description
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
									activePaintId
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
								editors {
									editor {
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
									}
									state
								}
								editableEmoteSetIds
								permissions {
									user {
										manageAny
										useCustomProfilePicture
									}
									emote {
										manageAny
									}
									emoteSet {
										manageAny
									}
									ticket {
										create
									}
								}
							}
						}
					}
				}
			`),
			{
				id: userId,
				paintId,
			},
		)
		.toPromise();

	if (!res.data) {
		throw res.error?.message;
	}

	return res.data.users.user.activePaint as User;
}

export async function setMainConnection(userId: string, platform: Platform, platformId: string) {
	const res = await gqlClient().mutation(
		graphql(`
			mutation SetMainConnection($userId: Id!, $platform: Platform!, $platformId: String!) {
				users {
					user(id: $userId) {
						mainConnection(platform: $platform, platformId: $platformId) {
							id
							mainConnection {
								platform
								platformId
								platformDisplayName
								platformAvatarUrl
							}
							connections {
								platform
								platformId
								platformDisplayName
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
								user {
									manageAny
									useCustomProfilePicture
								}
								emote {
									manageAny
								}
								emoteSet {
									manageAny
								}
								ticket {
									create
								}
							}
						}
					}
				}
			}
		`),
		{ userId, platform, platformId },
	);

	if (!res.data) {
		throw res.error?.message;
	}

	return res.data.users.user.mainConnection as User;
}

export async function uploadProfilePicture(userId: string, data: Blob) {
	const token = get(sessionToken);

	if (!token) {
		return undefined;
	}

	const response = await fetch(`${PUBLIC_REST_API_V4}/users/${userId}/profile-picture`, {
		method: "POST",
		body: data,
		credentials: "include",
		headers: {
			Authorization: `Bearer ${token}`,
		},
	});

	const json = await response.json();

	if (response.ok) {
		return json;
	} else {
		currentError.set(`Failed to upload profile picture: ${json.error}`);
		errorDialogMode.set("shown");
	}
}

export async function removeProfilePicture(userId: string) {
	const res = await gqlClient().mutation(
		graphql(`
			mutation RemoveProfilePicture($userId: Id!) {
				users {
					user(id: $userId) {
						removeProfilePicture {
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
								user {
									manageAny
									useCustomProfilePicture
								}
								emote {
									manageAny
								}
								emoteSet {
									manageAny
								}
								ticket {
									create
								}
							}
						}
					}
				}
			}
		`),
		{ userId },
	);

	if (!res.data) {
		return undefined;
	}

	return res.data.users.user.removeProfilePicture as User;
}

export async function removeConnection(userId: string, platform: Platform, platformId: string) {
	const res = await gqlClient().mutation(
		graphql(`
			mutation RemoveConnection($userId: Id!, $platform: Platform!, $platformId: String!) {
				users {
					user(id: $userId) {
						removeConnection(platform: $platform, platformId: $platformId) {
							id
							mainConnection {
								platform
								platformId
								platformDisplayName
								platformAvatarUrl
							}
							connections {
								platform
								platformId
								platformDisplayName
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
								user {
									manageAny
									useCustomProfilePicture
								}
								emote {
									manageAny
								}
								emoteSet {
									manageAny
								}
								ticket {
									create
								}
							}
						}
					}
				}
			}
		`),
		{ userId, platform, platformId },
	);

	if (!res.data) {
		return undefined;
	}

	return res.data.users.user.removeConnection as User;
}

export async function deleteAllSessions(userId: string) {
	const res = await gqlClient().mutation(
		graphql(`
			mutation DeleteAllSessions($userId: Id!) {
				users {
					user(id: $userId) {
						deleteAllSessions
					}
				}
			}
		`),
		{ userId },
	);

	if (!res.data) {
		return undefined;
	}

	return res.data.users.user.deleteAllSessions;
}
