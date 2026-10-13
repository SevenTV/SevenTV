<script lang="ts">
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import type { PageData } from "./$types";
	import type { Emote, EmoteSearchResult } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { emotesLayout } from "$/lib/layout";
	import ActiveEmoteSetButton from "$/components/users/active-emote-set-button.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import Button from "$/components/input/button.svelte";
	import Toggle from "$/components/input/toggle.svelte";
	import { Plant, Trophy } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { gql } from "@urql/svelte";

	let { data }: { data: PageData } = $props();

	type SortOption = "top" | "new";
	let sortBy = $state<SortOption>("top");
	let selectionMode = $state(false);
	let allEmotes = $state<Emote[] | null>(null);

	$effect(() => {
		data.id;
		$defaultEmoteSet;
		allEmotes = null;
	});

	async function fetchEmotes(): Promise<Emote[]> {
		if (allEmotes) return allEmotes;

		const variables = {
			id: data.id,
			isDefaultSetSet: !!$defaultEmoteSet,
			defaultSetId: $defaultEmoteSet ?? "",
		};

		const gql_query = gql`
			query UserUploadedEmotes($id: Id!, $isDefaultSetSet: Boolean!, $defaultSetId: Id!) {
				users {
					user(id: $id) {
						ownedEmotes {
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
							scores {
								topAllTime
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
			}
		`;

		const res = await gqlClient().query(gql_query, variables).toPromise();
		if (res.error || !res.data) {
			throw res.error;
		}
		const emotes = res.data.users.user?.ownedEmotes;
		if (!emotes) {
			throw new Error("No emotes found");
		}
		allEmotes = emotes as Emote[];
		return allEmotes;
	}

	async function load(_page: number, _perPage: number): Promise<EmoteSearchResult> {
		const emotes = await fetchEmotes();
		const sorted = [...emotes];

		if (sortBy === "top") {
			sorted.sort((a, b) => {
				const scoreDiff = (b.scores?.topAllTime ?? 0) - (a.scores?.topAllTime ?? 0);
				if (scoreDiff !== 0) return scoreDiff;
				return b.id.localeCompare(a.id);
			});
		} else {
			sorted.sort((a, b) => b.id.localeCompare(a.id));
		}

		return {
			items: sorted,
			totalCount: sorted.length,
			pageCount: 1,
		};
	}
</script>

<div class="nav-bar">
	<div class="nav-left-group">
		<ActiveEmoteSetButton bind:userData={data.streamed.userRequest.value} />
		<nav class="tabs">
			<TabLink
				title={$t("common.top")}
				matcher={() => sortBy === "top"}
				onclick={() => (sortBy = "top")}
				responsive
			>
				<Trophy />
				{#snippet active()}
					<Trophy weight="fill" />
				{/snippet}
			</TabLink>

			<TabLink
				title={$t("common.new")}
				matcher={() => sortBy === "new"}
				onclick={() => (sortBy = "new")}
				responsive
			>
				<Plant />
				{#snippet active()}
					<Plant weight="fill" />
				{/snippet}
			</TabLink>
		</nav>

		<Button secondary onclick={() => (selectionMode = !selectionMode)}>
			{$t("labels.select")}
			{#snippet iconRight()}
				<Toggle bind:value={selectionMode} />
			{/snippet}
		</Button>
	</div>

	<div class="buttons">
		<LayoutButtons bind:value={$emotesLayout} />
	</div>
</div>

{#key data.id + sortBy}
	<EmoteLoader bind:selectionMode {load} />
{/key}

<style lang="scss">
	.nav-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-wrap: wrap;
		gap: 1rem;
	}

	.nav-left-group {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		flex-wrap: wrap;
	}

	.tabs {
		display: flex;
		align-items: center;
		background-color: var(--bg-light);
		border-radius: 0.5rem;
		gap: 0.3rem;
		padding: 0.3rem;
	}

	.buttons {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
