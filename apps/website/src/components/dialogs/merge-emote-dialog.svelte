<script lang="ts">
	import type { Emote } from "$/gql/graphql";
	import { mergeEmote } from "$/lib/emoteMutations";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let targetEmoteId = $state("");
	let isValidEmoteId = $state(false);

	let { mode = $bindable("hidden"), data = $bindable() }: Props = $props();

	let loading = $state(false);

	const emoteIdRegex = /^[A-Z0-9]{26}$/;

	function validateEmoteId(id: string) {
		isValidEmoteId = emoteIdRegex.test(id);
	}

	async function click(targetId: string) {
		loading = true;

		const newData = await mergeEmote(data.id, targetId);

		if (newData) {
			loading = false;
			mode = "hidden";
		}
	}
</script>

{#snippet loadingSpinner()}
	<Spinner />
{/snippet}

<EmoteDialog
	width={35}
	title={$t("dialogs.merge_emote.title", { values: { name: data.defaultName } })}
	bind:mode
	{data}
>
	<input
		type="text"
		placeholder={$t("dialogs.merge_emote.input_placeholder")}
		class="input-text"
		bind:value={targetEmoteId}
		oninput={() => validateEmoteId(targetEmoteId)}
	/>
	<span class="details">{$t("dialogs.merge_emote.warning_message")}</span>
	{#snippet buttons()}
		<Button
			style="color: var(--danger)"
			submit
			disabled={loading || !isValidEmoteId}
			onclick={() => click(targetEmoteId)}
			icon={loading ? loadingSpinner : undefined}
		>
			{$t("pages.emote.merge")}
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
