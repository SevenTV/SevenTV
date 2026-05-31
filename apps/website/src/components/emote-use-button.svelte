<script lang="ts">
	import type { Emote, EmoteSet } from "$/gql/graphql";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { addEmoteToSet, removeEmoteFromSet } from "$/lib/setMutations";
	import { Broom, Minus, Plus } from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "./spinner.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";
	import { setDeficit } from "$/lib/emoteSetCapacity";
	import MakeRoomDialog from "./dialogs/make-room-dialog.svelte";
	import { type DialogMode } from "./dialogs/dialog.svelte";

	interface Props {
		data?: Emote;
		big?: boolean;
		primary?: boolean;
		oncomplete?: () => void;
	}

	let { data, big = false, primary = false, oncomplete }: Props = $props();

	let defaultSet = $derived(
		$defaultEmoteSet && data
			? $editableEmoteSets.find((s) => s.id === $defaultEmoteSet)
			: undefined,
	);

	let setName = $derived(defaultSet?.name);

	let active = $derived(
		defaultSet && data ? defaultSet.emotes.items.some((e) => e.id === data?.id) : undefined,
	);

	let loading = $state(false);

	let conflictingName = $derived(
		defaultSet && data
			? defaultSet.emotes.items.some((e) => e.alias === data?.defaultName)
			: undefined,
	);

	// Slots the default set is short by to add this emote (0 if it fits / already in / unlimited).
	let requiredSlots = $derived(defaultSet && data ? setDeficit(defaultSet, [data.id]) : 0);
	// Full for this emote: not already in it, no name conflict, and no free slot.
	let full = $derived(!active && !conflictingName && requiredSlots > 0);

	let makeRoomMode = $state<DialogMode>("hidden");
	let makeRoomSet = $state<EmoteSet | undefined>(undefined);

	function openMakeRoom() {
		if (!defaultSet) return;
		makeRoomSet = defaultSet;
		makeRoomMode = "shown";
	}

	$effect(() => {
		if (active === undefined) {
			return;
		}

		loading = false;

		// This gets called before the next effect runs which means it skips the first render
		return () => {
			oncomplete?.();
		};
	});

	async function useEmote() {
		if (!$defaultEmoteSet || !data) {
			return;
		}

		loading = true;
		await addEmoteToSet($defaultEmoteSet, data.id);
	}

	async function removeEmote() {
		if (!$defaultEmoteSet || !data) {
			return;
		}

		loading = true;
		await removeEmoteFromSet($defaultEmoteSet, data.id);
	}
</script>

{#if $defaultEmoteSet}
	{#if active}
		<Button onclick={removeEmote} disabled={loading} {big} {primary}>
			{#snippet icon()}
				{#if loading}
					<Spinner />
				{:else}
					<Minus />
				{/if}
			{/snippet}
			{$t("dialogs.emote_set.default_emote_set.remove", { values: { set: setName } })}
		</Button>
	{:else if full}
		<Button
			onclick={openMakeRoom}
			title={$t("dialogs.make_room.over_capacity", { values: { count: requiredSlots } })}
			{big}
			{primary}
		>
			{#snippet icon()}
				<Broom />
			{/snippet}
			{$t("dialogs.emote_set.default_emote_set.make_room", { values: { set: setName } })}
		</Button>
	{:else}
		<Button
			onclick={useEmote}
			title={conflictingName ? $t("dialogs.emote_set.default_emote_set.conflicting_name") : undefined}
			disabled={loading || conflictingName}
			{big}
			{primary}
		>
			{#snippet icon()}
				{#if loading}
					<Spinner />
				{:else}
					<Plus />
				{/if}
			{/snippet}
			{$t("dialogs.emote_set.default_emote_set.add", { values: { set: setName } })}
		</Button>
	{/if}
{/if}

{#if makeRoomSet}
	<MakeRoomDialog bind:mode={makeRoomMode} emoteSet={makeRoomSet} requiredSlots={requiredSlots} />
{/if}
