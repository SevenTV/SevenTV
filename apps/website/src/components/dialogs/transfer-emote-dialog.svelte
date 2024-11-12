<script lang="ts">
	import { User } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();
</script>

<EmoteDialog
	width={35}
	title={$t("dialogs.transfer_emote.title", { values: { emote: "AlienPls" } })}
	bind:mode
>
	<span class="details">
		{$t("dialogs.transfer_emote.details", { values: { emote: "AlienPls" } })}
	</span>
	<TextInput placeholder={$t("labels.search_users", { values: { count: 1 } })}>
		<span class="label">{$t("dialogs.transfer_emote.receipient")}</span>
		{#snippet icon()}
			<User />
		{/snippet}
	</TextInput>
	{#snippet buttons()}
		<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
		<Button primary submit>{$t("dialogs.transfer_emote.transfer")}</Button>
	{/snippet}
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
