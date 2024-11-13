import { graphql } from "$/gql";
import type { LayoutLoadEvent } from "./$types";
import type { Emote } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";

export async function load({ fetch, params }: LayoutLoadEvent) {
	// TODO: Don't do this in load function because it takes too long
	const req = gqlClient()
		.query(
			graphql(`
				query OneEmote($id: Id!) {
					emotes {
						emote(id: $id) {
							id
							defaultName
							owner {
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
							tags
							flags {
								defaultZeroWidth
								publicListed
							}
							attribution {
								user {
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
								}
							}
							images {
								url
								mime
								size
								width
								height
								scale
								frameCount
							}
							ranking(ranking: TRENDING_WEEKLY)
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
				console.error(res.error);
			}

			return res.data?.emotes.emote as Emote;
		});

	return {
		id: params.id,
		streamed: {
			emote: req,
		},
	};
}
