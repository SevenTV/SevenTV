<script lang="ts">
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import { graphql } from "$/gql";
	import type { PageData } from "./$types";
	import type { EmoteSetEmoteSearchResult } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";

	let { data }: { data: PageData } = $props();

	function load(page: number, _perPage: number): Promise<EmoteSetEmoteSearchResult> {
		return gqlClient()
			.query(
				graphql(`
					query UserActiveEmotes($id: Id!, $page: Int!) {
						users {
							user(id: $id) {
								style {
									activeEmoteSet {
										emotes(page: $page, perPage: 100) {
											__typename
											items {
												alias
												flags {
													zeroWidth
												}
												emote {
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
													flags {
														# animated
														# approvedPersonal
														defaultZeroWidth
														# deniedPersonal
														# nsfw
														# private
														publicListed
													}
													images {
														url
														mime
														size
														scale
														width
														frameCount
													}
													ranking(ranking: TRENDING_WEEKLY)
												}
											}
											totalCount
											pageCount
										}
									}
								}
							}
						}
					}
				`),
				{
					id: data.id,
					page,
				},
			)
			.toPromise()
			.then((res) => {
				if (res.error || !res.data) {
					throw res.error;
				}

				const emotes = res.data.users.user?.style.activeEmoteSet?.emotes;

				if (!emotes) {
					return {
						items: [],
						totalCount: 0,
						pageCount: 0,
					};
				}

				return emotes as EmoteSetEmoteSearchResult;
			});
	}
</script>

<div class="buttons">
	<LayoutButtons />
</div>
<EmoteLoader {load} />

<style lang="scss">
	.buttons {
		align-self: flex-end;

		display: flex;
		gap: 0.5rem;
	}
</style>
