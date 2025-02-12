<script lang="ts">
	import type { SubscriptionInfo, SubscriptionProduct } from "$/gql/graphql";
	import { ArrowSquareOut, CaretDown, Spinner, Ticket } from "phosphor-svelte";
	import StoreSection from "../store/store-section.svelte";
	import Button from "../input/button.svelte";
	import DropDown from "../drop-down.svelte";
	import { user } from "$/lib/auth";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { priceFormat, variantName } from "$/lib/utils";

	interface Props {
		subInfo?: SubscriptionInfo;
		product?: SubscriptionProduct;
	}

	let purchaseLoading = $state<string>();
	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);

	let { subInfo, product }: Props = $props();
	async function purchase(variantId?: string) {
		if (!$user) {
			return;
		}

		purchaseLoading = variantId ?? "pass";

		const pickemsId = "01JK6K2GCE06A9F6FSBVZA2KQA";

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation PurchasePickems(
						$userId: Id!
						$pickemsId: Id!
						$subscriptionPriceId: StripeProductId
					) {
						billing(userId: $userId) {
							getPickems(pickemsId: $pickemsId, subscriptionPriceId: $subscriptionPriceId) {
								checkoutUrl
							}
						}
					}
				`),
				{ userId: $user.id, pickemsId, subscriptionPriceId: variantId },
			)
			.toPromise();

		if (res.data) {
			window.location.href = res.data.billing.getPickems.checkoutUrl;
		}

		purchaseLoading = undefined;
	}
</script>

<StoreSection title="Pick'ems Pass">
	{#snippet header()}
		<div class="buttons">
			{#if !subInfo}
				<Button href="" disabled secondary style="color: var(--store)">
					{#snippet icon()}
						<Ticket />
					{/snippet}
					<span> Sign in to purchase </span>
				</Button>
			{:else if hasPass}
				<Button href="https://pickems.tv" secondary>
					{#snippet icon()}
						<ArrowSquareOut />
					{/snippet}
					<span> Place Pick'ems </span>
				</Button>
			{:else if subInfo.activePeriod}
				<Button onclick={() => purchase()} secondary style="color: var(--store)">
					{#snippet icon()}
						<Ticket />
					{/snippet}
					<span> Get Pass </span>
				</Button>
			{:else}
				<DropDown>
					{#snippet dropdown()}
						<Button
							big
							onclick={() => purchase()}
							disabled={purchaseLoading !== undefined}
							style="width: 100%"
						>
							{#snippet icon()}
								{#if purchaseLoading === "pass"}
									<Spinner />
								{:else}
									<Ticket />
								{/if}
							{/snippet}
							<div class="button-text">
								<span> Pick'ems Pass </span>
								<span>
									{priceFormat("eur").format(4.99)}
								</span>
							</div>
						</Button>
						{#each product?.variants ?? [] as variant}
							<Button
								big
								onclick={() => purchase(variant.id)}
								disabled={purchaseLoading !== undefined}
								style="width: 100%"
							>
								{#snippet icon()}
									{#if purchaseLoading === variant.id}
										<Spinner />
									{:else}
										<Ticket />
									{/if}
								{/snippet}
								<div class="button-text">
									<span>
										Pass + {variantName(variant)} Subscription Bundle
									</span>
									<span>
										{priceFormat("eur").format((variant.price.amount + 350) / 100)}
									</span>
								</div>
							</Button>
						{/each}
					{/snippet}
					<Button secondary hideOnMobile>
						{#snippet icon()}
							<Ticket weight="bold" />
						{/snippet}
						<span> Get Pass </span>
						{#snippet iconRight()}
							<CaretDown />
						{/snippet}
					</Button>
					<Button secondary hideOnDesktop>
						{#snippet icon()}
							<Ticket weight="bold" />
						{/snippet}
						{#snippet iconRight()}
							<CaretDown />
						{/snippet}
					</Button>
				</DropDown>
			{/if}
		</div>
	{/snippet}
</StoreSection>

<style lang="scss">
	.button-text {
		width: 100%;
		display: flex;
		justify-content: space-between;
		gap: 1rem;
	}
</style>
