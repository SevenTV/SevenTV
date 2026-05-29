<script lang="ts">
	import type {
		Emote,
		EmoteSearchResult,
		EmoteSetEmote,
		EmoteSetEmoteSearchResult,
		EmoteSetKind,
	} from "$/gql/graphql";
	import { emotesLayout } from "$/lib/layout";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteContainer from "./emote-container.svelte";
	import InfiniteLoading, { type InfiniteEvent } from "svelte-infinite-loading";
	import Spinner from "../spinner.svelte";
	import { untrack } from "svelte";
	import { t } from "svelte-i18n";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { type DialogMode } from "../dialogs/dialog.svelte";
	import { addEmoteToSet, removeEmoteFromSet } from "$/lib/setMutations";
	import MassEmoteSetPicker from "../mass-emote-set-picker.svelte";
	import Button from "../input/button.svelte";
	import Dialog from "../dialogs/dialog.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";

	const PER_PAGE = 72;

	interface Props {
		load: (page: number, perPage: number) => Promise<EmoteSearchResult | EmoteSetEmoteSearchResult>;
		scrollable?: boolean;
		setKind?: EmoteSetKind;
		selectionMode?: boolean;
		selectionMap?: { [key: string]: boolean };
		emoteSetName?: string;
		emoteSetId?: string;
		isDeletionMode?: boolean;
	}

	let {
		load,
		scrollable,
		setKind,
		selectionMode = $bindable(false),
		selectionMap = $bindable({}),
		emoteSetName,
		emoteSetId,
		isDeletionMode = $bindable(false),
	}: Props = $props();

	interface Results {
		items: { emote: Emote; emoteSetEmote?: EmoteSetEmote }[];
		pageCount: number;
		totalCount: number;
	}

	let page = $state(0);
	let results: Results | undefined = $state();
	let identifier = $state(0);
	let massEmoteSetPickerDialogMode = $state<DialogMode>("hidden");
	let selectedEmotesMap = $state<{ [key: string]: Emote }>({});
	let selectedEmotesArray = $derived(Object.values(selectedEmotesMap));
	let submitting = $state(false);
	let selectedCount = $derived(Object.keys(selectedEmotesMap).length);

	let alias = $state(selectedEmotesArray[0]?.defaultName ?? "");

	let originalState = $derived.by(() => {
		const state: { [key: string]: string | undefined } = {};

		for (const set of $editableEmoteSets) {
			state[set.id] = set.emotes.items.find(
				(e) => e.emote?.id === selectedEmotesArray[0]?.id,
			)?.alias;
		}

		return state;
	});
	let pickedEmoteSets: { [key: string]: boolean } = $state({});

	$effect(() => {
		pickedEmoteSets = Object.fromEntries(
			Object.entries(originalState).map(([k, v]) => [k, v === alias]),
		);
	});

	let toAdd = $derived(
		Object.keys(pickedEmoteSets).filter((k) => pickedEmoteSets[k] && !originalState[k]),
	);

	let toRemove = $derived(
		Object.keys(pickedEmoteSets).filter((k) => !pickedEmoteSets[k] && originalState[k] === alias),
	);

	let toRename = $derived(
		Object.keys(pickedEmoteSets).filter(
			(k) => pickedEmoteSets[k] && originalState[k] && originalState[k] !== alias,
		),
	);

	let sumChanges = $derived(toAdd.length + toRemove.length + toRename.length);

	function getEmoteKey(emote: Emote, emoteSetEmote?: EmoteSetEmote): string {
		return emoteSetEmote?.alias || emote.defaultName || emote.id;
	}

	// limit selection to 20 emotes to prevent rate limit issues
	function syncSelection(key: string, emote: Emote, isSelected: boolean): boolean {
		if (isSelected) {
			if (selectedCount >= 20) return false;
			selectedEmotesMap[key] = emote;
		} else {
			delete selectedEmotesMap[key];
		}
		selectedEmotesMap = selectedEmotesMap;
		return true;
	}

	function resetComponent() {
		// reload component (temporary solution until EmoteLoader get's a better reactive loading system)
		untrack(() => {
			reset();
		});
	}

	async function handleBulkDeleteAction() {
		if (isDeletionMode) {
			submitting = true;
			try {
				const itemsToRemove = results?.items.filter((item, index) => {
					const key = `${index}-${getEmoteKey(item.emote, item.emoteSetEmote)}`;
					return selectionMap[key];
				});
				if (itemsToRemove && emoteSetId) {
					await Promise.all(
						itemsToRemove.map((item) =>
							removeEmoteFromSet(
								emoteSetId,
								item.emoteSetEmote?.id || item.emote.id,
								item.emoteSetEmote?.alias,
							),
						),
					);
				}
				selectionMode = false;
				selectionMap = {};
				selectedEmotesMap = {};
			} catch (e) {
				console.error("Failed to remove emotes", e);
			} finally {
				submitting = false;
				resetComponent();
			}
		} else {
			massEmoteSetPickerDialogMode = "shown";
		}
	}

	// function toggleSelectionMode() {
	// 	selectionMode = !selectionMode;
	// 	if (!selectionMode) {
	// 		selectionMap = {};
	// 		selectedEmotesMap = {};
	// 	}
	// }

	async function submit(emotesToProcess: Emote[]) {
		const targetSetIds = Object.keys(pickedEmoteSets).filter((id) => pickedEmoteSets[id]);

		if (emotesToProcess.length === 0 || targetSetIds.length === 0) return;
		submitting = true;
		try {
			for (const emote of emotesToProcess) {
				const emoteId = emote.id;
				const targetAlias = alias || emote.defaultName || "";

				console.log(`Adding ${emote.defaultName} to ${targetSetIds.length} sets...`);
				await Promise.allSettled(
					targetSetIds.map((setId) => addEmoteToSet(setId, emoteId, targetAlias)),
				);
			}
			selectionMode = false;
			selectionMap = {};
			selectedEmotesMap = {};
			massEmoteSetPickerDialogMode = "hidden";
		} catch (error) {
			console.error("Mass add failed:", error);
		} finally {
			submitting = false;
		}
	}

	export function reset() {
		page = 0;
		results = undefined;
		identifier++;
	}

	$effect(() => {
		$emotesLayout;
		$defaultEmoteSet;
		resetComponent();
	});

	function handleInfinite(event: InfiniteEvent) {
		const currentIdentifier = identifier;

		load(++page, PER_PAGE)
			.then((result) => {
				if (currentIdentifier !== identifier) return;

				let newItems: { emote: Emote; emoteSetEmote?: EmoteSetEmote }[] = [];

				if (result.__typename === "EmoteSetEmoteSearchResult") {
					newItems = result.items
						.filter((e) => e.emote)
						.map((item) => ({
							emote: item.emote!,
							emoteSetEmote: item,
						}));
				} else {
					newItems = (result as EmoteSearchResult).items.map((item) => ({
						emote: item,
						emoteSetEmote: undefined,
					}));
				}

				const startIndex = results?.items.length ?? 0;

				if (results) {
					results.pageCount = result.pageCount;
					results.totalCount = result.totalCount;
					results.items.push(...newItems);
				} else {
					results = {
						items: newItems,
						pageCount: result.pageCount,
						totalCount: result.totalCount,
					};
				}

				for (let j = 0; j < newItems.length; j++) {
					const item = newItems[j];
					const globalIndex = startIndex + j;
					const key = `${globalIndex}-${getEmoteKey(item.emote, item.emoteSetEmote)}`;

					if (selectionMap[key] === undefined) {
						selectionMap[key] = false;
					}
				}

				if (results.items.length > 0) event.detail.loaded();
				if (results.pageCount <= page) event.detail.complete();
			})
			.catch(() => {
				event.detail.error();
			});
	}
</script>

<Dialog bind:mode={massEmoteSetPickerDialogMode} width={50}>
	<div class="dialog-inner">
		<div class="content-body">
			<MassEmoteSetPicker
				bind:value={pickedEmoteSets}
				emote={selectedEmotesArray[0]}
				{alias}
				hideEmoteDialogMode={() => (massEmoteSetPickerDialogMode = "hidden")}
				disabled={false}
				{toAdd}
				{toRemove}
				{toRename}
			/>
		</div>

		<div class="footer">
			<Button onclick={() => (massEmoteSetPickerDialogMode = "hidden")}>
				{$t("labels.cancel")}
			</Button>

			{#if submitting}
				<Button primary submit disabled>
					{#snippet iconRight()}
						<Spinner />
					{/snippet}
					{$t(sumChanges ? "dialogs.add_emote.confirm_changes" : "dialogs.add_emote.confirm", {
						values: { count: sumChanges },
					})}
				</Button>
			{:else}
				<Button
					primary
					submit
					onclick={() => submit(selectedEmotesArray)}
					disabled={toAdd.length === 0 && toRemove.length === 0 && toRename.length === 0}
				>
					{$t(sumChanges ? "dialogs.add_emote.confirm_changes" : "dialogs.add_emote.confirm", {
						values: { count: sumChanges },
					})}
				</Button>
			{/if}
		</div>
	</div>
</Dialog>

<div class="selection-container" class:expanded={selectionMode}>
	<div class="toolbar">
		{#if selectionMode}
			<div class="selection-controls">
				<span class="count">{selectedEmotesArray.length}{$t("dialogs.emote_set.mass_select_feature.amount_selected_label")}</span>

				<Button
					secondary={!isDeletionMode}
					disabled={selectedEmotesArray.length === 0 || submitting}
					onclick={handleBulkDeleteAction}
					style="background-color: var(--bg-medium); border: none;"
				>
					{#if submitting}
						<Spinner />
					{:else if isDeletionMode}
						{$t("dialogs.emote_set.mass_select_feature.remove")}
					{:else}
						{$t("dialogs.emote_set.mass_select_feature.add")}
					{/if}
				</Button>
			</div>
		{/if}
	</div>
</div>

<EmoteContainer {scrollable} layout={$emotesLayout} style="flex-grow: 1">
	{#if results}
		{#each results.items as data, i (i + "-" + getEmoteKey(data.emote, data.emoteSetEmote))}
			<EmotePreview
				data={data.emote}
				emoteInSet={data.emoteSetEmote}
				emoteSetEmote={data.emoteSetEmote}
				{setKind}
				index={i}
				emoteOnly={$emotesLayout === "small-grid"}
				{selectionMode}
				bind:selected={selectionMap[i + "-" + getEmoteKey(data.emote, data.emoteSetEmote)]}
				{emoteSetName}
				{emoteSetId}
				{resetComponent}
				onSelectionChange={(val: boolean) => {
					const key = i + "-" + getEmoteKey(data.emote, data.emoteSetEmote);
					if (val && selectedCount >= 20 && !selectedEmotesMap[key]) {
						selectionMap[key] = false; // Keep it false
						return;
					}
					syncSelection(key, data.emote, val);
				}}
			/>
		{/each}
	{/if}
	<div class="loading">
		<InfiniteLoading
			distance={500}
			identifier={{ identifier, layout: $emotesLayout }}
			on:infinite={handleInfinite}
		>
			<p slot="noMore">{$t("common.no_more_emotes")}</p>
			<p slot="noResults">{$t("common.no_emotes")}</p>
			<Spinner slot="spinner" />
		</InfiniteLoading>
	</div>
</EmoteContainer>

<style lang="scss">
	.selection-container {
		position: sticky;
		top: 0;
		transition: all 0.2s ease-in-out;
		height: 0rem;
		display: flex;
		align-items: center;
		padding: 0 1rem;
		z-index: 10;

		&.expanded {
			height: 2rem;
		}

		.toolbar {
			display: flex;
			align-items: center;
			width: 100%;
			gap: 1rem;
		}

		.selection-controls {
			display: flex;
			align-items: center;
			gap: 1.5rem;
			flex-grow: 1;

			.count {
				font-size: 0.9rem;
				font-weight: 600;
				color: #eee;
				background-color: var(--bg-medium);
				padding: 0.4rem 1.5rem;
				border-radius: 0.5rem;

			}
		}
	}

	.loading {
		grid-column: 1 / -1;
		align-self: start;
		width: 100%;
		height: 1rem;
	}

	.dialog-inner {
		display: flex;
		flex-direction: column;
		height: 40rem;
		padding: 1.5rem;
		box-sizing: border-box;
	}

	.content-body {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		margin-bottom: 1rem;
	}

	.footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		flex-shrink: 0;
	}
</style>
