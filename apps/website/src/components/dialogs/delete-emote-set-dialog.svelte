<script lang="ts">
	import type { EmoteSet } from "$/gql/graphql";
	import { deleteSet } from "$/lib/setMutations";
	import { goto } from "$app/navigation";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		data: EmoteSet;
	}

	let { mode = $bindable("hidden"), data = $bindable() }: Props = $props();

	let loading = $state(false);

	async function submit() {
		loading = true;

		await deleteSet(data.id);

		loading = false;
		mode = "hidden";

		goto("/");
	}
</script>

<Dialog width={40} bind:mode>
	<form class="layout">
		<div class="preview">
			<EmoteSetPreview {data} />
		</div>
		<div class="content">
			<h1>{$t("dialogs.delete_emote_or_set.title", { values: { name: data.name } })}</h1>
			<span class="details">{$t("dialogs.delete_emote_or_set.warning_message_set")}</span>
			<!-- <TextInput placeholder={$t("dialogs.delete_emote_or_set.reason")}>
				<span class="label">{$t("dialogs.delete_emote_or_set.reason_for_deletion")}</span>
			</TextInput> -->
			<div class="buttons">
				{#snippet loadingSpinner()}
					<Spinner />
				{/snippet}
				<Button
					style="color: var(--danger)"
					submit
					disabled={loading}
					onclick={submit}
					icon={loading ? loadingSpinner : undefined}
				>
					{$t("labels.delete")}
				</Button>
				<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			</div>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1.5rem 1rem;

		display: flex;
		gap: 2rem;
	}

	.preview {
		align-self: center;

		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.content {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.details {
		flex-grow: 1;

		color: var(--text-light);
		font-size: 0.875rem;
	}

	// .label {
	// 	font-size: 0.875rem;
	// 	font-weight: 500;
	// }

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
