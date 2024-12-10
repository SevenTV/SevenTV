import { graphql } from "$/gql";
import type { User } from "$/gql/graphql";
import { gqlClient } from "./gql";

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
								}
								emote {
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
									}
									emote {
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
									}
									emote {
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
