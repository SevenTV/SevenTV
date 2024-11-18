import { graphql } from "$/gql";
import type { LayoutLoadEvent } from "./$types";
import type { Emote } from "$/gql/graphql";
import { gqlClient } from "$/lib/gql";
import { get } from "svelte/store";
import { defaultEmoteSet } from "$/lib/defaultEmoteSet";

export async function load({ fetch, params }: LayoutLoadEvent) {
	const defaultSet = get(defaultEmoteSet);

	const req = gqlClient()
		.query(
			graphql(`
				query OneEmote($id: Id!, $isDefaultSetSet: Boolean!, $defaultSetId: Id!) {
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
							inEmoteSets(emoteSetIds: [$defaultSetId]) @include(if: $isDefaultSetSet) {
								emoteSetId
								emote {
									id
									alias
								}
							}
						}
					}
				}
			`),
			{
				id: params.id,
				isDefaultSetSet: !!defaultSet,
				defaultSetId: defaultSet ?? "",
			},
			{
				fetch,
			},
		)
		.toPromise()
		.then((res) => res.data?.emotes.emote as Emote);

	return {
		id: params.id,
		streamed: {
			emote: req,
		},
	};
}
