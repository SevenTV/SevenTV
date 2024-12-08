<script lang="ts">
	import type { Emote } from "$/gql/graphql";
	import { deleteEmote } from "$/lib/emoteMutations";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data = $bindable() }: Props = $props();

	let loading = $state(false);

	async function click() {
		loading = true;

		const newData = await deleteEmote(data.id);

		if (newData) {
			data = newData;
		}

		loading = false;
		mode = "hidden";
	}
</script>

{#snippet loadingSpinner()}
	<Spinner />
{/snippet}

<EmoteDialog
	width={35}
	title={$t("dialogs.delete_emote_or_set.title", { values: { name: data.defaultName } })}
	bind:mode
	{data}
>
	<span class="details">{$t("dialogs.delete_emote_or_set.warning_message_emote")}</span>
	{#snippet buttons()}
		<Button
			style="color: var(--danger)"
			submit
			disabled={loading}
			onclick={click}
			icon={loading ? loadingSpinner : undefined}
		>
			{$t("labels.delete")}
		</Button>
		<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
	{/snippet}
</EmoteDialog>

<style lang="scss">
	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	// .label {
	// 	font-size: 0.875rem;
	// 	font-weight: 500;
	// }
</style>
