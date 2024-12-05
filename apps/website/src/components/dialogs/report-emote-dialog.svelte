<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import { t } from "svelte-i18n";
	import { gqlClient } from "$/lib/gql";
	import Radio from "../input/radio.svelte";
	import { graphql } from "$/gql";
	import Spinner from "../spinner.svelte";

	const reasons = [
		"dialogs.report_emote.reasons.my_work",
		"dialogs.report_emote.reasons.duplicate",
		"dialogs.report_emote.reasons.sexual",
		"dialogs.report_emote.reasons.violance",
		"dialogs.report_emote.reasons.its_me",
		"dialogs.report_emote.reasons.offensive",
		"dialogs.report_emote.reasons.other",
	];

	let { mode = $bindable("hidden"), targetId }: { mode: DialogMode; targetId: string } = $props();

	let reason = $state<string>();
	let additionalInfo = $state("");

	let loading = $state(false);

	async function submit() {
		if (!reason) return;

		loading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation ReportEmote($targetId: Id!, $title: String!, $content: String) {
						tickets {
							createAbuseTicket(
								target: { kind: EMOTE, id: $targetId }
								title: $title
								content: $content
							) {
								id
							}
						}
					}
				`),
				{
					targetId,
					title: $t(reason, { locale: "en" }),
					content: additionalInfo.length ? additionalInfo : undefined,
				},
			)
			.toPromise();

		if (res.data?.tickets.createAbuseTicket.id) {
			loading = false;
			mode = "hidden";
		}
	}
</script>

<Dialog width={40} bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.report_emote.title")}</h1>
		<hr />
		<div class="reasons">
			<span class="label">{$t("dialogs.report_emote.choose_reasons")}</span>
			{#each reasons as reasonId}
				<Radio option name="reason" value={reasonId} bind:group={reason}>
					{#snippet leftLabel()}
						<span class="label">{$t(reasonId)}</span>
					{/snippet}
				</Radio>
			{/each}
		</div>
		<TextInput type="textarea" placeholder={$t("labels.enter_text")} bind:value={additionalInfo}>
			<span class="label">{$t("dialogs.report_emote.additional_info")}</span>
		</TextInput>
		<span class="details">{$t("dialogs.report_emote.disclaimer")}</span>
		<div class="buttons">
			<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			{#snippet loadingSpinner()}
				<Spinner />
			{/snippet}
			<Button
				primary
				submit
				disabled={!reason || loading}
				onclick={submit}
				icon={loading ? loadingSpinner : undefined}>{$t("labels.report")}</Button
			>
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
