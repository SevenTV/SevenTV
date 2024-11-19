<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Emote } from "$/gql/graphql";
	import Spinner from "../spinner.svelte";
	import { addEmoteToSet, removeEmoteFromSet } from "$/lib/setMutations";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data }: Props = $props();

	let alias = $state(data.defaultName);

	let originalState = $derived.by(() => {
		const state: { [key: string]: boolean } = {};
		const emoteAlias = alias;

		for (const set of $editableEmoteSets) {
			state[set.id] = set.emotes.items.some((e) => e.id === data.id && e.alias === emoteAlias);
		}

		return state;
	});

	let pickedEmoteSets: { [key: string]: boolean } = $state({});

	$effect(() => {
		pickedEmoteSets = originalState;
	});

	let toAdd = $derived(
		Object.keys(pickedEmoteSets).filter((k) => pickedEmoteSets[k] && !originalState[k]),
	);

	let toRemove = $derived(
		Object.keys(pickedEmoteSets).filter((k) => !pickedEmoteSets[k] && originalState[k]),
	);

	let submitting = $state(false);

	async function submit() {
		submitting = true;

		for (const setId of toAdd) {
			await addEmoteToSet(setId, data.id, alias);
		}

		for (const setId of toRemove) {
			await removeEmoteFromSet(setId, data.id, alias);
		}

		mode = "hidden";
	}
</script>

{#snippet buttons()}
	<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
	{#if submitting}
		<Button primary submit disabled>
			{#snippet iconRight()}
				<Spinner />
			{/snippet}
			{$t("labels.confirm")}
		</Button>
	{:else}
		<Button primary submit onclick={submit} disabled={toAdd.length === 0 && toRemove.length === 0}>
			{$t("labels.confirm")}
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
		highlightAdd={toAdd}
		highlightRemove={toRemove}
	/>
</EmoteDialog>

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
