<script lang="ts">
	import { User } from "phosphor-svelte";
	import TagsInput from "../input/tags-input.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import TextInput from "../input/text-input.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.edit_emote.title", { values: { emote: "AlienPls" } })}</h1>
		<hr />
		<TextInput placeholder={$t("labels.emote_name")}>
			<span class="label">{$t("labels.emote_name")}</span>
		</TextInput>
		<div class="tags">
			<TagsInput>
				<span class="label">{$t("labels.tags")}</span>
			</TagsInput>
		</div>
		<TextInput placeholder={$t("labels.search_users", { values: { count: 2 } })}>
			<span class="label">{$t("labels.emote_attribution")}</span>
			{#snippet icon()}
				<User />
			{/snippet}
		</TextInput>
		<div>
			<span class="label">{$t("common.settings")}</span>
			<div class="settings">
				<Checkbox>{$t("flags.overlaying")}</Checkbox>
				<Checkbox>{$t("flags.listed")}</Checkbox>
				<Checkbox>{$t("flags.personal_use")}</Checkbox>
			</div>
		</div>
		<div class="buttons">
			<Button style="color: var(--danger); margin-right: auto;">{$t("labels.delete")}</Button>
			<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			<Button primary submit>{$t("labels.save")}</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.tags {
		display: flex;
		flex-direction: column;
	}

	.settings {
		margin-top: 0.4rem;

		display: grid;
		grid-template-columns: auto auto;
		gap: 0.5rem;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
