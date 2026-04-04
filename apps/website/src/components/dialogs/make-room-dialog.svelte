<script lang="ts">
	import { untrack } from "svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import TextInput from "../input/text-input.svelte";
	import Select, { type Option } from "../input/select.svelte";
	import ResponsiveImage from "../responsive-image.svelte";
	import Spinner from "../spinner.svelte";
	import { MagnifyingGlass } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { removeEmoteFromSet } from "$/lib/setMutations";
	import {
		fetchEmoteSetEmotesWithScores,
		type MakeRoomEmote,
		type TimeWindow,
	} from "$/lib/makeRoomQuery";
	import type { EmoteSet } from "$/gql/graphql";

	interface Props {
		mode: DialogMode;
		emoteSet: EmoteSet;
	}

	let { mode = $bindable("hidden"), emoteSet }: Props = $props();

	let loading = $state(true);
	let emotes = $state<MakeRoomEmote[]>([]);
	let selectedIds = $state<Set<string>>(new Set());
	let searchQuery = $state("");
	let timeWindow = $state<string>("topWeekly");
	let sortDirection = $state<string>("least");
	let removing = $state(false);

	const timeWindowOptions: Option[] = [
		{ value: "topDaily", label: "Today" },
		{ value: "topWeekly", label: "This Week" },
		{ value: "topMonthly", label: "This Month" },
		{ value: "topAllTime", label: "All Time" },
	];

	const sortOptions: Option[] = [
		{ value: "least", label: "Least Popular" },
		{ value: "most", label: "Most Popular" },
	];

	const timeWindowLabels: Record<string, string> = {
		topDaily: "today",
		topWeekly: "this week",
		topMonthly: "this month",
		topAllTime: "all time",
	};

	let prevMode: DialogMode | undefined = $state(undefined);

	$effect(() => {
		const currentMode = mode;
		const prev = untrack(() => prevMode);

		untrack(() => {
			prevMode = currentMode;
		});

		// Only reset when transitioning from hidden to visible
		if (currentMode !== "hidden" && (prev === "hidden" || prev === undefined)) {
			loading = true;
			selectedIds = new Set();
			searchQuery = "";
			removing = false;

			fetchEmoteSetEmotesWithScores(emoteSet.id).then((result) => {
				emotes = result;
				loading = false;
			});
		}
	});

	let filteredEmotes = $derived.by(() => {
		const query = searchQuery.toLowerCase();
		const tw = timeWindow as TimeWindow;
		const asc = sortDirection === "least";

		return emotes
			.filter((item) => {
				if (!query) return true;
				return (
					item.alias.toLowerCase().includes(query) ||
					item.emote.defaultName.toLowerCase().includes(query)
				);
			})
			.toSorted((a, b) =>
				asc ? a.emote.scores[tw] - b.emote.scores[tw] : b.emote.scores[tw] - a.emote.scores[tw],
			);
	});

	let selectedCount = $derived(selectedIds.size);

	function toggleSelection(emoteId: string) {
		const next = new Set(selectedIds);
		if (next.has(emoteId)) {
			next.delete(emoteId);
		} else {
			next.add(emoteId);
		}
		selectedIds = next;
	}

	function getScoreDisplay(item: MakeRoomEmote): string {
		const score = item.emote.scores[timeWindow as TimeWindow];
		const period = timeWindowLabels[timeWindow];
		const prefix = score >= 0 ? "+" : "";
		return `${prefix}${score.toLocaleString()} channels ${period}`;
	}

	let confirmDialogMode: DialogMode = $state("hidden");
	let modeBeforeConfirm: DialogMode = $state("shown");

	function promptRemove() {
		modeBeforeConfirm = mode;
		mode = "shown-without-close";
		confirmDialogMode = "shown";
	}

	// Restore make-room dialog when confirmation closes
	$effect(() => {
		if (confirmDialogMode === "hidden" && mode === "shown-without-close") {
			mode = modeBeforeConfirm;
		}
	});

	async function executeRemove() {
		removing = true;

		for (const emoteId of selectedIds) {
			await removeEmoteFromSet(emoteSet.id, emoteId);
		}

		confirmDialogMode = "hidden";
		mode = "hidden";
	}
</script>

<Dialog width={40} bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.make_room.title", { values: { set: emoteSet.name } })}</h1>
		<hr />

		<div class="controls">
			<TextInput placeholder={$t("dialogs.make_room.search_placeholder")} bind:value={searchQuery}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput>
			<Select options={sortOptions} bind:selected={sortDirection} />
			<Select options={timeWindowOptions} bind:selected={timeWindow} />
		</div>

		{#if loading}
			<div class="loading">
				<Spinner />
			</div>
		{:else}
			<div class="emote-list">
				{#each filteredEmotes as item (item.emote.id)}
					<button
						class="emote-row"
						class:selected={selectedIds.has(item.emote.id)}
						onclick={() => toggleSelection(item.emote.id)}
					>
						<Checkbox value={selectedIds.has(item.emote.id)} />
						<div class="emote-image">
							{#if item.emote.images.length > 0}
								<ResponsiveImage images={item.emote.images} width={32} height={32} />
							{:else}
								<div class="emote-placeholder"></div>
							{/if}
						</div>
						<span class="emote-name">{item.alias}</span>
						<span class="emote-score">{getScoreDisplay(item)}</span>
					</button>
				{/each}
			</div>
		{/if}

		<div class="buttons">
			<Button style="color: var(--danger)" disabled={selectedCount === 0} onclick={promptRemove}>
				{$t("dialogs.make_room.remove_selected", { values: { count: selectedCount } })}
			</Button>
			<Button secondary onclick={() => (mode = "hidden")}>
				{$t("labels.cancel")}
			</Button>
		</div>
	</div>
</Dialog>

<Dialog width={25} bind:mode={confirmDialogMode}>
	<div class="confirm-layout">
		<h2>{$t("dialogs.make_room.confirm_remove", { values: { count: selectedCount } })}</h2>
		<p class="confirm-text">
			This will remove {selectedCount} emote{selectedCount !== 1 ? "s" : ""} from {emoteSet.name}.
		</p>
		<div class="buttons">
			<Button style="color: var(--danger)" onclick={executeRemove} disabled={removing}>
				{#snippet icon()}
					{#if removing}
						<Spinner />
					{/if}
				{/snippet}
				{#if removing}
					{$t("dialogs.make_room.removing")}
				{:else}
					Remove
				{/if}
			</Button>
			<Button secondary onclick={() => (confirmDialogMode = "hidden")} disabled={removing}>
				{$t("labels.cancel")}
			</Button>
		</div>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		padding: 1.5rem 1rem;
	}

	h1 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0;
	}

	hr {
		border: none;
		border-top: 1px solid var(--border);
		margin: 0;
	}

	.controls {
		display: flex;
		gap: 0.5rem;
		align-items: center;

		& > :global(:first-child) {
			flex: 1;
		}
	}

	.loading {
		display: flex;
		justify-content: center;
		align-items: center;
		padding: 3rem 0;
	}

	.emote-list {
		max-height: 25rem;
		overflow-y: auto;
		scrollbar-gutter: stable;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.emote-row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 0.75rem;
		border-radius: 0.5rem;
		background-color: var(--bg-medium);
		border: 1px solid transparent;
		cursor: pointer;
		transition: border-color 0.1s;
		color: var(--text);
		font-size: 0.875rem;
		text-align: left;

		&:hover {
			border-color: var(--border-active);
		}

		&.selected {
			border-color: var(--primary);
		}
	}

	.emote-image {
		width: 2rem;
		height: 2rem;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;

		& > :global(picture) {
			width: 100%;
			height: 100%;
		}

		& > :global(picture > img) {
			object-fit: contain;
			width: 100%;
			height: 100%;
		}
	}

	.emote-placeholder {
		width: 2rem;
		height: 2rem;
		border-radius: 0.25rem;
		background-color: var(--preview);
	}

	.emote-name {
		font-weight: 500;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.emote-score {
		color: var(--text-light);
		font-size: 0.75rem;
		flex-shrink: 0;
	}

	.buttons {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
	}

	.confirm-layout {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		padding: 1.5rem 1rem;
	}

	.confirm-layout h2 {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0;
	}

	.confirm-text {
		font-size: 0.875rem;
		color: var(--text-light);
		margin: 0;
	}
</style>
