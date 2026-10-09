<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { Gift, PencilSimple, TextAlignLeft } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import TagsInput from "../input/tags-input.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Select from "../input/select.svelte";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import type { CreateRedeemCodeBatchInput, CreateRedeemCodeInput } from "$/gql/graphql";
	import { t } from "svelte-i18n";

	let { mode = $bindable() }: { mode: DialogMode } = $props();

	let number = $state(1);

	let name = $state("");

	let autoGenerate = $state(true);
	let code = $state("");

	let description = $state("");

	let tags = $state<string[]>([]);

	let uses = $state(1);

	let noExpiration = $state(true);
	let startDate = $state<string>("");
	let endDate = $state<string>("");

	let specialEventId = $state<string>("");

	let trialDays = $state(0);

	function reset() {
		mode = "hidden";

		number = 1;
		name = "";
		autoGenerate = true;
		code = "";
		description = "";
		tags = [];
		uses = 1;
		noExpiration = true;
		startDate = "";
		endDate = "";
		specialEventId = "";
		trialDays = 0;
	}

	$effect(() => {
		if (number > 1) {
			autoGenerate = true;
		}
	});

	let loading = $state(false);

	function download(filename: string, text: string) {
		const element = document.createElement("a");
		element.setAttribute("href", "data:text/plain;charset=utf-8," + encodeURIComponent(text));
		element.setAttribute("download", filename);

		element.style.display = "none";
		document.body.appendChild(element);

		element.click();

		document.body.removeChild(element);
	}

	async function submit() {
		if (
			loading ||
			!name ||
			(!autoGenerate && !code) ||
			!specialEventId ||
			(!noExpiration && (!startDate || !endDate))
		)
			return;

		loading = true;

		let commonData = {
			name,
			description,
			tags,
			uses,
			activePeriod:
				noExpiration || !startDate || !endDate
					? undefined
					: {
							start: new Date(startDate),
							end: new Date(endDate),
						},
			specialEventId,
			subscriptionEffect: trialDays
				? {
						productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
						trialDays,
						noRedirectToStripe: false,
					}
				: undefined,
		};

		if (number === 1) {
			let data: CreateRedeemCodeInput = commonData;

			if (!autoGenerate) {
				data.code = code;
			}

			const res = await gqlClient().mutation(
				graphql(`
					mutation AdminCreateRedeemCode($data: CreateRedeemCodeInput!) {
						redeemCodes {
							create(data: $data) {
								code
							}
						}
					}
				`),
				{ data },
			);

			if (res.data) {
				download("redeem-codes.txt", res.data.redeemCodes.create.code);
			}
		} else {
			const data: CreateRedeemCodeBatchInput = { number, ...commonData };

			const res = await gqlClient().mutation(
				graphql(`
					mutation AdminCreateRedeemCodes($data: CreateRedeemCodeBatchInput!) {
						redeemCodes {
							createBatch(data: $data) {
								code
							}
						}
					}
				`),
				{ data },
			);

			if (res.data) {
				download(
					"redeem-codes.txt",
					res.data.redeemCodes.createBatch.map((c) => c.code).join("\n"),
				);
			}
		}

		loading = false;
		reset();
	}

	let title = $derived($t("pages.admin.codes.create_dialog.title", { values: { number, codes: number === 1 ? $t("pages.admin.codes.create_dialog.code_singular") : $t("pages.admin.codes.create_dialog.code_plural") } }));

	async function querySpecialEvents() {
		const res = await gqlClient().query(
			graphql(`
				query AdminCreateRedeemCodeSpecialEvents {
					specialEvents {
						specialEvents {
							id
							name
						}
					}
				}
			`),
			{},
		);

		return res.data?.specialEvents.specialEvents.toReversed();
	}

	let specialEvents = $state(querySpecialEvents());
</script>

<Dialog width={35} bind:mode>
	<form class="layout">
		<h1>{title}</h1>
		<hr />
		<label>
			{$t("pages.admin.codes.create_dialog.number_of_codes")}
			<input type="number" required min="1" bind:value={number} />
		</label>
		<Checkbox bind:value={autoGenerate} disabled={number > 1}>{$t("pages.admin.codes.create_dialog.generate_random")}</Checkbox>
		{#if !autoGenerate}
			<TextInput
				placeholder={$t("pages.admin.codes.attributes.name.code")}
				bind:value={code}
				minlength={1}
				maxlength={24}
				required
				style="font-family: monospace;"
			>
				{#snippet icon()}
					<Gift />
				{/snippet}
				{$t("pages.admin.codes.attributes.name.code")}
			</TextInput>
		{/if}
		<label>
			{$t("pages.admin.special_events.header")}
			{#await specialEvents}
				<Spinner />
			{:then specialEvents}
				{#if specialEvents}
					<Select
						options={specialEvents.map((e) => {
							return {
								value: e.id,
								label: e.name,
							};
						})}
						bind:selected={specialEventId}
					/>
				{:else}
					<p>{$t("pages.admin.codes.create_dialog.no_events")}</p>
				{/if}
			{/await}
		</label>
		<label>
			{$t("pages.admin.codes.create_dialog.trial_days")}
			<input type="number" required min="0" bind:value={trialDays} />
		</label>
		<label>
			{$t("pages.admin.codes.create_dialog.uses_per_code")}
			<input type="number" required min="0" bind:value={uses} style="width: 6rem;" />
		</label>
		<Checkbox bind:value={noExpiration}>{$t("pages.admin.codes.create_dialog.always_active")}</Checkbox>
		{#if !noExpiration}
			{@const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone}
			<label>
				{$t("pages.admin.codes.create_dialog.start_date", { values: { timezone }})}
				<input type="datetime-local" required bind:value={startDate} />
			</label>
			<label>
				{$t("pages.admin.codes.create_dialog.end_date", { values: { timezone }})}
				<input type="datetime-local" required bind:value={endDate} />
			</label>
		{/if}
		<TextInput placeholder={$t("pages.admin.codes.attributes.name.name")} bind:value={name} required>
			{#snippet icon()}
				<PencilSimple />
			{/snippet}
			{$t("pages.admin.codes.attributes.name.name")}
		</TextInput>
		<TextInput placeholder={$t("pages.admin.codes.attributes.name.description")} bind:value={description}>
			{#snippet icon()}
				<TextAlignLeft />
			{/snippet}
			{$t("pages.admin.codes.attributes.name.description")}
		</TextInput>
		<div class="tags">
			<TagsInput bind:tags>{$t("pages.admin.codes.attributes.name.tags")}</TagsInput>
		</div>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		<div class="buttons">
			{#await specialEvents}
				<Button primary disabled>{title}</Button>
			{:then _}
				<Button
					primary
					icon={loading ? loadingSpinner : undefined}
					onclick={submit}
					disabled={loading ||
						!name ||
						(!autoGenerate && !code) ||
						!specialEventId ||
						(!noExpiration && (!startDate || !endDate))}
				>
					{title}
				</Button>
			{/await}
			<Button secondary onclick={reset}>{$t("labels.cancel")}</Button>
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

	.tags {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.buttons {
		display: flex;
		gap: 1rem;
	}
</style>
