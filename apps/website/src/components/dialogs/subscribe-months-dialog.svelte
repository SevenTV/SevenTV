<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";
	import { Minus, Plus, Star } from "phosphor-svelte";
	import { type SubscriptionProductVariant } from "$/gql/graphql";
	import { priceFormat } from "$/lib/utils";
	import { user } from "$/lib/auth";

	interface Props {
		mode: DialogMode;
		variant: SubscriptionProductVariant;
	}

	let { mode = $bindable("hidden"), variant }: Props = $props();

	// Keep in sync with `MAX_MONTHS` in apps/api/src/stripe_common.rs
	const MAX_MONTHS = 24;

	let months = $state(1);
	let subscribeLoading = $state(false);

	function incrementMonths() {
		months = Math.min(months + 1, MAX_MONTHS);
	}

	function decrementMonths() {
		months = Math.max(months - 1, 1);
	}

	let totalPrice = $derived(
		priceFormat(variant.price.currency).format((variant.price.amount / 100) * months),
	);

	async function subscribe() {
		if (!$user) {
			return;
		}

		subscribeLoading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation SubscribeMonths($userId: Id!, $variantId: StripeProductId!, $months: Int) {
						billing(userId: $userId) {
							subscribe(variantId: $variantId, months: $months) {
								checkoutUrl
							}
						}
					}
				`),
				{ userId: $user.id, variantId: variant.id, months },
			)
			.toPromise();

		if (res.data) {
			window.location.href = res.data.billing.subscribe.checkoutUrl;
		}

		subscribeLoading = false;
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.subscribe_months.title")}</h1>
		<hr />

		<div class="months-row">
			<span>{$t("dialogs.gift.months_label")}</span>
			<div class="stepper">
				<Button secondary onclick={decrementMonths} disabled={months <= 1}>
					{#snippet icon()}
						<Minus />
					{/snippet}
				</Button>
				<span class="months-value">{months}</span>
				<Button secondary onclick={incrementMonths} disabled={months >= MAX_MONTHS}>
					{#snippet icon()}
						<Plus />
					{/snippet}
				</Button>
			</div>
		</div>

		<p class="hint">
			{#if months === 1}
				{$t("dialogs.subscribe_months.auto_renew_hint")}
			{:else}
				{$t("dialogs.subscribe_months.prepaid_hint")}
			{/if}
		</p>

		<p class="total">
			{$t("dialogs.gift.total")}: <strong>{totalPrice}</strong>
		</p>

		<div class="buttons">
			<Button secondary onclick={() => (mode = "hidden")} style="margin-right: auto">
				{$t("labels.cancel")}
			</Button>

			{#snippet spinnerIcon()}
				<Spinner />
			{/snippet}
			{#snippet starIcon()}
				<Star />
			{/snippet}

			<Button
				icon={subscribeLoading ? spinnerIcon : starIcon}
				disabled={subscribeLoading}
				onclick={subscribe}
				primary
				submit
			>
				{$t("dialogs.buttons.continue")}
			</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		height: 100%;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.months-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.stepper {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.months-value {
		min-width: 1.5rem;
		text-align: center;
		font-weight: 600;
	}

	.total {
		font-size: 0.875rem;
	}

	.hint {
		color: var(--text-light);
		font-size: 0.8rem;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
