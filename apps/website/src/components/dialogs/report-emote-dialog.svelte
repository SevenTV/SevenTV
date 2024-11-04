<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import { t } from "svelte-i18n";

	const reasons = [
		$t("dialogs.report_emote.reasons.my_work"),
		$t("dialogs.report_emote.reasons.duplicate"),
		$t("dialogs.report_emote.reasons.sexual"),
		$t("dialogs.report_emote.reasons.violance"),
		$t("dialogs.report_emote.reasons.its_me"),
		$t("dialogs.report_emote.reasons.offensive"),
		$t("dialogs.report_emote.reasons.other"),
	];

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();
</script>

<Dialog width={40} bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.report_emote.title")}</h1>
		<hr />
		<div class="reasons">
			<span class="label">{$t("dialogs.report_emote.choose_reasons")}</span>
			{#each reasons as reason}
				<Checkbox option>
					{#snippet leftLabel()}
						<span class="label">{reason}</span>
					{/snippet}
				</Checkbox>
			{/each}
		</div>
		<TextInput type="textarea" placeholder={$t("labels.enter_text")}>
			<span class="label">{$t("dialogs.report_emote.additional_info")}</span>
		</TextInput>
		<span class="details">{$t("dialogs.report_emote.disclaimer")}</span>
		<div class="buttons">
			<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			<Button primary submit>{$t("labels.report")}</Button>
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

	.details {
		font-size: 0.75rem;
		color: var(--text-light);
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.reasons {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.buttons {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 0.5rem;
	}
</style>
