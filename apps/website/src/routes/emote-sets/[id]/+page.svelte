<script lang="ts">
	import Tags from "$/components/emotes/tags.svelte";
	import type { PageData } from "./$types";
	import Flags, { emoteSetToFlags } from "$/components/flags.svelte";
	import EditEmoteSetDialog from "$/components/dialogs/edit-emote-set-dialog.svelte";
	import { type DialogMode } from "$/components/dialogs/dialog.svelte";
	import CopyEmotesDialog from "$/components/dialogs/copy-emotes-dialog.svelte";
	import RemoveEmotesDialog from "$/components/dialogs/remove-emotes-dialog.svelte";
	import { t } from "svelte-i18n";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import EmoteLoader from "$/components/layout/emote-loader.svelte";
	import type { EmoteSetEmoteSearchResult } from "$/gql/graphql";
	import Button from "$/components/input/button.svelte";
	import Toggle from "$/components/input/toggle.svelte";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";

	let { data }: { data: PageData } = $props();

	// let enabled = $state(false);
	let selectionMode = $state(false);
	let selectionMap = $state({});
	let editDialogMode: DialogMode = $state("hidden");
	let copyEmotesDialogMode: DialogMode = $state("hidden");
	let removeEmotesDialogMode: DialogMode = $state("hidden");

	async function queryEmotes(page: number, perPage: number) {
		const res = await gqlClient().query(
			graphql(`
				query EmotesInSet(
					$id: Id!
					$page: Int!
					$perPage: Int!
					$isDefaultSetSet: Boolean!
					$defaultSetId: Id!
				) {
					emoteSets {
						emoteSet(id: $id) {
							emotes(page: $page, perPage: $perPage) {
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
			`),
			{
				id: data.emoteSet.id,
				page,
				perPage,
				isDefaultSetSet: !!$defaultEmoteSet,
				defaultSetId: $defaultEmoteSet ?? "",
			},
		);

		if (res.error || !res.data) {
			throw res.error;
		}

		const emotes = res.data.emoteSets.emoteSet?.emotes;

		if (!emotes) {
			throw new Error("No emotes found");
		}

		return emotes as EmoteSetEmoteSearchResult;
	}
</script>

<svelte:head>
	<title>{data.emoteSet.name} - {$t("page_titles.suffix")}</title>
</svelte:head>

<EditEmoteSetDialog bind:mode={editDialogMode} />
<CopyEmotesDialog bind:mode={copyEmotesDialogMode} />
<RemoveEmotesDialog bind:mode={removeEmotesDialogMode} />
<div class="layout">
	<div class="set-info">
		<h1>{data.emoteSet.name}</h1>
		<Flags
			flags={emoteSetToFlags(data.emoteSet)}
			style="position: absolute; top: 1rem; right: 1rem;"
		/>
		<Tags tags={data.emoteSet.tags} />
		{#if data.emoteSet.capacity}
			<div class="progress">
				<progress value={data.emoteSet.emotes.totalCount} max={data.emoteSet.capacity}></progress>
				{data.emoteSet.emotes.totalCount}/{data.emoteSet.capacity}
			</div>
		{/if}
	</div>
	<div class="controls">
		<div class="buttons">
			<Button secondary onclick={() => (selectionMode = !selectionMode)} hideOnDesktop>
				{$t("labels.select")}
				{#snippet iconRight()}
					<Toggle bind:value={selectionMode} />
				{/snippet}
			</Button>
			<!-- <HideOn mobile={selectionMode}>
				<Button primary onclick={() => (enabled = !enabled)}>
					{#if enabled}
						{$t("labels.disable")}
					{:else}
						{$t("labels.enable")}
					{/if}
					{#snippet iconRight()}
						{#if enabled}
							<LightningSlash />
						{:else}
							<Lightning />
						{/if}
					{/snippet}
				</Button>
			</HideOn>
			<Button secondary hideOnMobile onclick={() => (editDialogMode = "shown")}>
				{$t("labels.edit")}
				{#snippet iconRight()}
					<NotePencil />
				{/snippet}
			</Button>
			<Button secondary hideOnMobile>
				{$t("pages.emote_set.copy_set")}
				{#snippet iconRight()}
					<Copy />
				{/snippet}
			</Button>
			{#if !selectionMode}
				<Button secondary hideOnDesktop onclick={() => (editDialogMode = "shown")}>
					{#snippet iconRight()}
						<NotePencil />
					{/snippet}
				</Button>
				<Button secondary hideOnDesktop>
					{#snippet iconRight()}
						<Copy />
					{/snippet}
				</Button>
			{/if} -->
			<Button secondary onclick={() => (selectionMode = !selectionMode)} hideOnMobile>
				{$t("labels.selection_mode")}
				{#snippet iconRight()}
					<Toggle bind:value={selectionMode} />
				{/snippet}
			</Button>
			<!-- {#if selectionMode}
				<Button onclick={() => (copyEmotesDialogMode = "shown")}>
					{#snippet icon()}
						<Copy />
					{/snippet}
				</Button>
				<Button>
					{#snippet icon()}
						<NotePencil />
					{/snippet}
				</Button>
				<Button onclick={() => (removeEmotesDialogMode = "shown")}>
					{#snippet icon()}
						<Trash />
					{/snippet}
				</Button>
			{/if} -->
		</div>
		<div class="buttons">
			<!-- <Select
				options={[
					{ value: "none", label: $t("labels.no_filters") },
					{ value: "filters", label: $t("labels.filters") },
				]}
			/>
			<TextInput placeholder={$t("labels.search")}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput> -->
			<LayoutButtons />
		</div>
	</div>
	<div class="content">
		<EmoteLoader load={queryEmotes} scrollable={false} {selectionMode} bind:selectionMap />
	</div>
</div>

<style lang="scss">
	.layout {
		padding: 1.25rem;
		padding-bottom: 0;
		height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.set-info {
		position: relative;
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.75rem;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		h1 {
			text-align: center;
			font-size: 1.125rem;
			font-weight: 500;
		}

		.progress {
			display: flex;
			align-items: center;
			gap: 0.75rem;

			font-size: 0.875rem;
			font-weight: 500;

			progress {
				flex-grow: 1;
			}
		}
	}

	.controls {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap-reverse;
		justify-content: space-between;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.content {
		flex-grow: 1;

		overflow: auto;
		scrollbar-gutter: stable;
		margin-right: -1.25rem;
		padding-right: 1.25rem;
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
			padding-bottom: 0;
		}

		.content {
			margin-right: 0;
			padding-right: 0;
		}
	}
</style>
