<script lang="ts">
	import type { Emote } from "$/gql/graphql";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { addEmoteToSet, removeEmoteFromSet } from "$/lib/setMutations";
	import { Minus, Plus } from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import Spinner from "./spinner.svelte";
	import { editableEmoteSets } from "$/lib/emoteSets";

	interface Props {
		data?: Emote;
		big?: boolean;
		primary?: boolean;
		oncomplete?: () => void;
	}

	let { data, big = false, primary = false, oncomplete }: Props = $props();

	let active = $derived(
		($defaultEmoteSet && data)
			? $editableEmoteSets
					.find((s) => s.id === $defaultEmoteSet)
					?.emotes.items.some((e) => e.id === data?.id)
			: undefined,
	);

	let loading = $state(false);

	let conflictingName = $derived(($defaultEmoteSet && data)
			? $editableEmoteSets
					.find((s) => s.id === $defaultEmoteSet)
					?.emotes.items.some((e) => e.alias === data?.defaultName)
			: undefined,
		);

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
			Remove Emote
		</Button>
	{:else}
		<Button onclick={useEmote} title={conflictingName ? "Conflicting Name" : undefined} disabled={loading || conflictingName} {big} {primary}>
			{#snippet icon()}
				{#if loading}
					<Spinner />
				{:else}
					<Plus />
				{/if}
			{/snippet}
			Use Emote
		</Button>
	{/if}
{/if}
