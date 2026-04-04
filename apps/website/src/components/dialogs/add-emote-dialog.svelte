<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import MakeRoomDialog from "./make-room-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Emote, EmoteSet } from "$/gql/graphql";
	import Spinner from "../spinner.svelte";
	import { Broom } from "phosphor-svelte";
	import { addEmoteToSet, removeEmoteFromSet, renameEmoteInSet } from "$/lib/setMutations";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data }: Props = $props();

	let alias = $state(data.defaultName);

	let originalState = $derived.by(() => {
		const state: { [key: string]: string | undefined } = {};

		for (const set of $editableEmoteSets) {
			state[set.id] = set.emotes.items.find((e) => e.id === data.id)?.alias;
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

	let submitting = $state(false);

	function isSetFull(setId: string): boolean {
		const set = $editableEmoteSets.find((s) => s.id === setId);
		return !!(
			set &&
			set.capacity &&
			set.emotes.totalCount >= set.capacity &&
			!originalState[setId]
		);
	}

	// When a full set gets selected, deselect all non-full sets (and vice versa)
	let prevPicked: { [key: string]: boolean } = {};

	$effect(() => {
		const justSelected = Object.keys(pickedEmoteSets).find(
			(k) => pickedEmoteSets[k] && !prevPicked[k],
		);

		if (justSelected && isSetFull(justSelected)) {
			// Deselect all other sets
			for (const k of Object.keys(pickedEmoteSets)) {
				if (k !== justSelected && pickedEmoteSets[k]) {
					pickedEmoteSets[k] = false;
				}
			}
		} else if (justSelected && !isSetFull(justSelected)) {
			// Deselect any full sets
			for (const k of Object.keys(pickedEmoteSets)) {
				if (k !== justSelected && pickedEmoteSets[k] && isSetFull(k)) {
					pickedEmoteSets[k] = false;
				}
			}
		}

		prevPicked = { ...pickedEmoteSets };
	});

	// Reset state when dialog opens
	$effect(() => {
		if (mode !== "hidden") {
			submitting = false;
			prevPicked = {};
			pickedEmoteSets = Object.fromEntries(
				Object.entries(originalState).map(([k, v]) => [k, v === alias]),
			);
		}
	});

	// Find selected sets that are full (user checked them but they're at capacity)
	let selectedFullSets = $derived(
		$editableEmoteSets.filter(
			(set) =>
				pickedEmoteSets[set.id] &&
				!originalState[set.id] &&
				set.capacity &&
				set.emotes.totalCount >= set.capacity,
		),
	);

	let hasSelectedFullSet = $derived(selectedFullSets.length > 0);
	let lockedSetId = $derived(selectedFullSets.length > 0 ? selectedFullSets[0].id : undefined);

	let makeRoomMode: DialogMode = $state("hidden");
	let makeRoomSet: EmoteSet | undefined = $state(undefined);

	function openMakeRoom() {
		if (selectedFullSets.length > 0) {
			makeRoomSet = selectedFullSets[0];
			// Lock the parent dialog so mouseTrap doesn't close it
			mode = "shown-without-close";
			makeRoomMode = "shown";
		}
	}

	// When make-room dialog closes, restore parent dialog to normal
	$effect(() => {
		if (makeRoomMode === "hidden" && mode === "shown-without-close") {
			mode = "shown";
		}
	});

	async function submit() {
		submitting = true;

		for (const setId of toAdd) {
			await addEmoteToSet(setId, data.id, alias);
		}

		for (const setId of toRemove) {
			await removeEmoteFromSet(setId, data.id);
		}

		for (const setId of toRename) {
			await renameEmoteInSet(setId, data.id, alias);
		}

		mode = "hidden";
	}
</script>

{#snippet buttons()}
	<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
	{#if hasSelectedFullSet}
		<Button primary onclick={openMakeRoom}>
			{#snippet icon()}
				<Broom />
			{/snippet}
			{$t("dialogs.make_room.button")}
		</Button>
	{:else if submitting}
		<Button primary submit disabled>
			{#snippet iconRight()}
				<Spinner />
			{/snippet}
			{#if sumChanges}
				{$t("dialogs.add_emote.confirm_changes", { values: { count: sumChanges } })}
			{:else}
				{$t("dialogs.add_emote.confirm")}
			{/if}
		</Button>
	{:else}
		<Button
			primary
			submit
			onclick={submit}
			disabled={toAdd.length === 0 && toRemove.length === 0 && toRename.length === 0}
		>
			{#if sumChanges}
				{$t("dialogs.add_emote.confirm_changes", { values: { count: sumChanges } })}
			{:else}
				{$t("dialogs.add_emote.confirm")}
			{/if}
		</Button>
	{/if}
{/snippet}

<EmoteDialog
	title={$t("dialogs.add_emote.title", { values: { emote: data.defaultName } })}
	bind:mode
	{buttons}
	{data}
>
	{#snippet preview()}
		<TextInput
			placeholder={$t("labels.emote_name")}
			style="max-width: 12.5rem"
			disabled={submitting}
			bind:value={alias}
		>
			<span class="label">{$t("dialogs.add_emote.change_name")}</span>
		</TextInput>
	{/snippet}
	<EmoteSetPicker
		bind:value={pickedEmoteSets}
		disabled={submitting}
		{toAdd}
		{toRemove}
		{toRename}
		emote={data}
		{alias}
		{lockedSetId}
	/>
</EmoteDialog>

{#if makeRoomSet}
	<MakeRoomDialog bind:mode={makeRoomMode} emoteSet={makeRoomSet} />
{/if}

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
