import { graphql } from "$/gql";
import type { LayoutLoadEvent } from "./$types";
import type { User } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";
import { ProxyState } from "$/lib/proxy.svelte";

export function load({ fetch, params }: LayoutLoadEvent) {
	const req = gqlClient()
		.query(
			graphql(`
				query OneUser($id: Id!) {
					users {
						user(id: $id) {
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
								activeEmoteSet {
									id
									name
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
								editorId
								permissions {
									emoteSet {
										create
									}
									user {
										manageProfile
										manageEditors
									}
								}
								state
							}
						}
					}
				}
			`),
			{
				id: params.id,
			},
			{
				fetch,
			},
		)
		.toPromise()
		.then((res) => {
			if (res.error || !res.data) {
				throw "Failed to load user";
			}

			if (!res.data.users.user) {
				throw "User not found";
			}

			return res.data.users.user as User;
		});

	const state = new ProxyState(req);

	return {
		id: params.id,
		streamed: {
			userRequest: state,
		},
	};
}
