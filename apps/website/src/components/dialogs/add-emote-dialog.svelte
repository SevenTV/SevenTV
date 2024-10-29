<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import { t } from "svelte-i18n";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<EmoteDialog title={$t("dialogs.add_emote.title", { values: { emote: "AlienPls" } })} bind:mode>
	<TextInput placeholder={$t("labels.emote_name")} style="max-width: 12.5rem" slot="preview">
		<span class="label">{$t("dialogs.add_emote.change_name")}</span>
	</TextInput>
	<EmoteSetPicker />
	<svelte:fragment slot="buttons">
		<Button on:click={() => (mode = DialogMode.Hidden)}>{$t("labels.cancel")}</Button>
		<Button primary submit>{$t("labels.confirm")}</Button>
	</svelte:fragment>
</EmoteDialog>

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
