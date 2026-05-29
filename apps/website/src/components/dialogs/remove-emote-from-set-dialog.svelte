<script lang="ts">
	import type { Emote, EmoteSetEmote } from "$/gql/graphql";
	import { removeEmoteFromSet } from "$/lib/setMutations";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		data: Emote;
		emoteInSet?: EmoteSetEmote;
		emoteSetName?: string;
		resetComponent?: () => void;
		emoteSetId?: string;
	}

	let {
		mode = $bindable("hidden"),
		data = $bindable(),
		emoteInSet = $bindable(),
		emoteSetId = $bindable(),
		resetComponent,
		emoteSetName = $bindable(),
	}: Props = $props();

	let targetEmoteName = $state("");
	targetEmoteName = emoteInSet?.alias ?? data.defaultName;

	let loading = $state(false);

	// used try catch for future mutation/query functions remake throwing errors
	async function click() {
		loading = true;
		try {
			await removeEmoteFromSet(emoteSetId ?? "", data.id, targetEmoteName);
			resetComponent?.();
			mode = "hidden";
		} catch (error) {
			console.error("Failed to rename emote:", error);
		} finally {
			loading = false;
		}
	}
</script>

{#snippet loadingSpinner()}
	<Spinner />
{/snippet}

<EmoteDialog
	width={35}
	title={$t("dialogs.remove_emote_from_set.title", {
		values: { set: emoteSetName ?? "" },
	})}
	bind:mode
	{data}
>
	<span class="details"
		>{$t("dialogs.remove_emote_from_set.warning_message", {
			values: { emote: data.defaultName, set: emoteSetName ?? "" },
		})}</span
	>
	{#snippet buttons()}
		<Button
			style="color: var(--danger)"
			submit
			disabled={loading}
			onclick={click}
			icon={loading ? loadingSpinner : undefined}
		>
			{$t("dialogs.remove_emote_from_set.confirm")}
		</Button>
		<Button
			secondary
			onclick={() => {
				mode = "hidden";
			}}>{$t("labels.cancel")}</Button
		>
	{/snippet}
</EmoteDialog>

<style lang="scss">
	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}
</style>
