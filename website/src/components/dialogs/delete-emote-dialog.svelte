<script lang="ts">
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import { DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<EmoteDialog width={35} title={$t("dialogs.delete_emote_or_set.title", { values: { name: "AlienPls" } })} bind:mode>
	<span class="details">{$t("dialogs.delete_emote_or_set.warning_message_emote")}</span>
	<TextInput placeholder={$t("dialogs.delete_emote_or_set.reason")}>
		<span class="label">{$t("dialogs.delete_emote_or_set.reason_for_deletion")}</span>
	</TextInput>
	<svelte:fragment slot="buttons">
		<Button style="color: var(--danger)" submit>{$t("labels.delete")}</Button>
		<Button secondary on:click={() => (mode = DialogMode.Hidden)}>{$t("labels.cancel")}</Button>
	</svelte:fragment>
</EmoteDialog>

<style lang="scss">
	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
