<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";
	import type { Emote } from "$/gql/graphql";

	interface Props {
		mode: DialogMode;
		data: Emote;
		buttons?: Snippet;
	}

	let { mode = $bindable("hidden"), data, buttons = fallbackButtons }: Props = $props();
</script>

{#snippet fallbackButtons()}
	<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
	<Button primary submit>{$t("labels.confirm")}</Button>
{/snippet}

<EmoteDialog
	title={$t("dialogs.add_emote.title", { values: { emote: data.defaultName } })}
	bind:mode
	{buttons}
	{data}
>
	{#snippet preview()}
		<TextInput placeholder={$t("labels.emote_name")} style="max-width: 12.5rem">
			<span class="label">{$t("dialogs.add_emote.change_name")}</span>
		</TextInput>
	{/snippet}
	<EmoteSetPicker />
</EmoteDialog>

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
