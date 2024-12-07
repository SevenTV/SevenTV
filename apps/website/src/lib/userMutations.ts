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
