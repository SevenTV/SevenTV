<script lang="ts">
	import { Warning } from "phosphor-svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import { t } from "svelte-i18n";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();

	const reasons = [
		$t("dialogs.delete_account.reasons.no_longer_use"),
		$t("dialogs.delete_account.reasons.privacy_concerns"),
		$t("dialogs.delete_account.reasons.not_useful"),
		$t("dialogs.delete_account.reasons.other"),
	];
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("common.delete_account")}</h1>
		<hr />
		<div class="warning">
			<Warning size="4rem" weight="fill" color="var(--danger)" />
			<h2>{$t("dialogs.delete_account.warning")}</h2>
			<span class="details">{$t("dialogs.delete_account.warning_message")}</span>
		</div>
		<div class="reasons">
			<span class="label">{$t("dialogs.delete_account.choose_reasons")}</span>
			{#each reasons as reason}
				<Checkbox option>
					{#snippet leftLabel()}
						<span class="label">
							{reason}
						</span>
					{/snippet}
				</Checkbox>
			{/each}
		</div>
		<TextInput placeholder="ayyybubu">
			<span class="label">{$t("dialogs.delete_account.confirm_username")}</span>
		</TextInput>
		<div class="buttons">
			<Button style="color: var(--danger)" submit>{$t("labels.delete")}</Button>
			<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
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

	h2 {
		font-size: 1.5rem;
		font-weight: 500;
	}

	.details {
		font-size: 0.875rem;
		color: var(--text-light);
	}

	.warning {
		display: flex;
		flex-direction: column;
		align-items: center;

		text-align: center;
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
