<script lang="ts">
	import Button from "../input/button.svelte";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import TextInput from "../input/text-input.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog width={40} bind:mode>
	<form class="layout">
		<div class="preview">
			<EmoteSetPreview />
		</div>
		<div class="content">
			<h1>{$t("dialogs.delete_emote_or_set.title", { values: { name: "Cat Emotes" } })}</h1>
			<span class="details">{$t("dialogs.delete_emote_or_set.warning_message_set")}</span>
			<TextInput placeholder={$t("dialogs.delete_emote_or_set.reason")}>
				<span class="label">{$t("dialogs.delete_emote_or_set.reason_for_deletion")}</span>
			</TextInput>
			<div class="buttons">
				<Button style="color: var(--danger)" submit>{$t("labels.delete")}</Button>
				<Button secondary on:click={() => (mode = DialogMode.Hidden)}>{$t("labels.cancel")}</Button>
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
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
