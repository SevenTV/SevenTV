<script lang="ts">
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import type { PageData } from "./$types";
	import type { EmoteSetEmoteSearchResult } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import { emotesLayout } from "$/lib/layout";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import ActiveEmoteSetButton from "$/components/users/active-emote-set-button.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { MagnifyingGlass } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { gql } from "@urql/svelte";

	let { data }: { data: PageData } = $props();
	let query: string = $state("");

	let timeout: NodeJS.Timeout | number | undefined;

	async function load(page: number, _perPage: number): Promise<EmoteSetEmoteSearchResult> {
		const search = query || undefined;

		const variables = {
			userId: data.id,
			page: page,
			query: search,
			isDefaultSetSet: !!$defaultEmoteSet,
			defaultSetId: $defaultEmoteSet ?? "",
		};

		const gql_query = gql`
			query SearchEmotesInActiveSet(
				$userId: Id!
				$query: String
				$page: Int!
				$isDefaultSetSet: Boolean!
				$defaultSetId: Id!
			) {
				users {
					user(id: $userId) {
						style {
							activeEmoteSet {
								id
								emotes(query: $query, page: $page, perPage: 100) {
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
												private
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
											inEmoteSets(emoteSetIds: [$defaultSetId]) @include(if: $isDefaultSetSet) {
												emoteSetId
												emote {
													id
													alias
												}
											}
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
		`;

		return new Promise((resolve, reject) => {
			if (timeout) {
				clearTimeout(timeout);
			}

			timeout = setTimeout(async () => {
				try {
					const res = await gqlClient().query(gql_query, variables).toPromise();

					if (res.error || !res.data) {
						reject(res.error ?? new Error("No data returned"));
					}

					resolve(res.data.users.user?.style.activeEmoteSet?.emotes);
				} catch (error) {
					console.error("Failed to load emotes:", error);
					reject(error);
				}
			}, 200);
		});
	}
</script>

<div class="buttons">
	<ActiveEmoteSetButton bind:userData={data.streamed.userRequest.value} />
	<div class="layout-buttons">
		<TextInput placeholder={$t("labels.search")} bind:value={query}>
			{#snippet icon()}
				<MagnifyingGlass />
			{/snippet}
		</TextInput>
		<LayoutButtons bind:value={$emotesLayout} />
	</div>
</div>
{#key data.id + query}
	<EmoteLoader {load} />
{/key}

<style lang="scss">
	.buttons {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		justify-content: space-between;
	}

	.layout-buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
