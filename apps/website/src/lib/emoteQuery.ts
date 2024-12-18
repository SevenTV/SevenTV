import { graphql } from "$/gql";
import type { EmoteSearchResult, Filters, SortBy } from "$/gql/graphql";
import { gqlClient } from "./gql";
import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
import { get } from "svelte/store";

let timeout: NodeJS.Timeout | number | undefined;

export async function queryEmotes(
	query: string | null,
	tags: string[],
	sort: SortBy,
	filters: Filters | null,
	page: number | null,
	perPage: number,
): Promise<EmoteSearchResult> {
	// Small timeout to prevent spamming requests when user is typing

	return new Promise((resolve, reject) => {
		if (timeout) {
			clearTimeout(timeout);
		}

		timeout = setTimeout(async () => {
			const defaultSet = get(defaultEmoteSet);

			const res = await gqlClient()
				.query(
					graphql(`
						query EmoteSearch(
							$query: String
							$tags: [String!]!
							$sortBy: SortBy!
							$filters: Filters
							$page: Int
							$perPage: Int!
							$isDefaultSetSet: Boolean!
							$defaultSetId: Id!
						) {
							emotes {
								search(
									query: $query
									tags: { tags: $tags, match: ANY }
									sort: { sortBy: $sortBy, order: DESCENDING }
									filters: $filters
									page: $page
									perPage: $perPage
								) {
									items {
										id
										defaultName
										owner {
											mainConnection {
												platformDisplayName
											}
											style {
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
										deleted
										flags {
											# animated
											# approvedPersonal
											defaultZeroWidth
											# deniedPersonal
											# nsfw
											# private
											publicListed
										}
										imagesPending
										images {
											url
											mime
											size
											scale
											width
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
									totalCount
									pageCount
								}
							}
						}
					`),
					{
						query,
						tags,
						sortBy: sort,
						filters,
						page,
						perPage,
						isDefaultSetSet: !!defaultSet,
						defaultSetId: defaultSet ?? "",
					},
				)
				.toPromise();

			if (res.error || !res.data) {
				reject(res.error);
			} else {
				resolve(res.data.emotes.search as EmoteSearchResult);
			}
		}, 200);
	});
}
