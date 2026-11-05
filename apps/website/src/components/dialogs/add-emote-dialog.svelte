<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Emote, EmoteSet } from "$/gql/graphql";
	import Spinner from "../spinner.svelte";
	import { addEmoteToSet, removeEmoteFromSet, renameEmoteInSet } from "$/lib/setMutations";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";
	import MakeRoomDialog from "./make-room-dialog.svelte";
	import { setDeficit } from "$/lib/emoteSetCapacity";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data }: Props = $props();

	let alias = $state(data.defaultName);

	let originalState = $derived.by(() => {
		const state: { [key: string]: string | undefined } = {};

		for (const set of $editableEmoteSets) {
			state[set.id] = set.emotes.items.find((e) => e.emote?.id === data?.id)?.alias;
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
		$editableEmoteSets
			.filter(
				(s) => pickedEmoteSets[s.id] && !originalState[s.id] && setDeficit(s, [data.id]) === 0,
			)
			.map((s) => s.id),
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

	// Selected sets that still cannot fit this emote (skipped by Confirm until Make Room is used).
	let blockedSets = $derived(
		$editableEmoteSets.filter((s) => pickedEmoteSets[s.id] && setDeficit(s, [data.id]) > 0),
	);

	let makeRoomMode: DialogMode = $state("hidden");
	let makeRoomSet: EmoteSet | undefined = $state(undefined);
	let makeRoomRequiredSlots = $state(0);
	let makeRoomOpen = $state(false);

	function openMakeRoom(set: EmoteSet, requiredSlots: number) {
		makeRoomSet = set;
		makeRoomRequiredSlots = requiredSlots;
		// Lock the parent dialog so its mouse-trap doesn't close it behind the child.
		mode = "shown-without-close";
		makeRoomOpen = true;
		makeRoomMode = "shown";
	}

	// When the make-room dialog closes, restore the parent dialog to normal.
	$effect(() => {
		if (makeRoomOpen && makeRoomMode === "hidden") {
			makeRoomOpen = false;
			mode = "shown";
		}
	});

	async function submit() {
		if (submitting) return;
		submitting = true;

		for (const setId of toAdd) {
			await addEmoteToSet(setId, data.id, alias);
		}

		for (const setId of toRemove) {
			let currentEditingSet = $editableEmoteSets.find((set) => set.id === setId);
			let emoteInSet = currentEditingSet?.emotes.items.find((e) => e.emote.id === data.id);
			if (emoteInSet) {
				await removeEmoteFromSet(
					setId,
					currentEditingSet?.emotes.items.find((e) => e.emote.id === data.id)?.id || data.id,
				);
			}
		}

		for (const setId of toRename) {
			await renameEmoteInSet(setId, data.id, alias);
		}

		submitting = false;
		if (blockedSets.length === 0) {
			mode = "hidden";
		}
	}

	function hide() {
		makeRoomOpen = false;
		makeRoomMode = "hidden";
		makeRoomSet = undefined;
		mode = "hidden";
	}
</script>

{#snippet buttons()}
	{#if blockedSets.length > 0}
		<span class="blocked-note">{$t("dialogs.make_room.not_added")}</span>
	{/if}
	<Button onclick={hide}>{$t("labels.cancel")}</Button>
	{#if submitting}
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
	width={50}
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
		hideEmoteDialogMode={hide}
		{alias}
		onMakeRoom={openMakeRoom}
	/>
</EmoteDialog>

{#if makeRoomSet}
	<MakeRoomDialog bind:mode={makeRoomMode} emoteSet={makeRoomSet} requiredSlots={makeRoomRequiredSlots} />
{/if}

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.blocked-note {
		font-size: 0.8125rem;
		color: var(--admin-unlist);
		margin-right: auto;
	}
</style>
