<script lang="ts">
	import Button from "../input/button.svelte";
	import SubInfo from "../sub-info.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretDown, DotsThreeVertical, Gift, PaintBrush, Star, Warning } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import DropDown from "../drop-down.svelte";
	import {
		SubscriptionState,
		type SubscriptionInfo,
		type SubscriptionProduct,
	} from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import { user } from "$/lib/auth";
	import Spinner from "../spinner.svelte";

	let { subInfo, product }: { subInfo: SubscriptionInfo; product: SubscriptionProduct } = $props();

	let renewSubLoading = $state(false);

	async function renewSubscription() {
		if (!$user) {
			return;
		}

		renewSubLoading = true;

		await gqlClient()
			.mutation(
				graphql(`
					mutation RenewSubscription($userId: Id!, $productId: Id!) {
						billing(userId: $userId) {
							renewSubscription(productId: $productId)
						}
					}
				`),
				{
					userId: $user.id,
					productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
				},
			)
			.toPromise();

		renewSubLoading = false;
	}

	let cancelSubLoading = $state(false);

	async function cancelSubscription() {
		if (!$user) {
			return;
		}

		cancelSubLoading = true;

		await gqlClient()
			.mutation(
				graphql(`
					mutation CancelSubscription($userId: Id!, $productId: Id!) {
						billing(userId: $userId) {
							cancelSubscription(productId: $productId)
						}
					}
				`),
				{
					userId: $user.id,
					productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
				},
			)
			.toPromise();

		cancelSubLoading = false;
	}
</script>

<StoreSection title={subInfo.activePeriod ? $t("common.your_subscription") : "Become a subscriber"}>
	{#snippet header()}
		<div class="buttons">
			{#if subInfo.activePeriod}
				<Button secondary hideOnMobile style="color: var(--store)">
					{#snippet icon()}
						<Star weight="fill" />
					{/snippet}
					<span>
						{$t("pages.store.subscription.subscribed")}
					</span>
				</Button>
				<Button secondary hideOnDesktop style="color: var(--store)">
					{#snippet icon()}
						<Star weight="fill" />
					{/snippet}
				</Button>
			{:else}
				<DropDown>
					{#snippet dropdown()}
						{#each product.variants as variant}
							<Button big>
								{#snippet icon()}
									<Star />
								{/snippet}
								{variant.kind} ({variant.price.amount}
								{variant.price.currency})
							</Button>
						{/each}
					{/snippet}
					<Button secondary hideOnMobile>
						{#snippet icon()}
							<Star weight="bold" />
						{/snippet}
						<span>
							{$t("pages.store.subscription.subscribe")}
						</span>
						{#snippet iconRight()}
							<CaretDown />
						{/snippet}
					</Button>
					<Button secondary hideOnDesktop>
						{#snippet icon()}
							<Star weight="bold" />
						{/snippet}
						{#snippet iconRight()}
							<CaretDown />
						{/snippet}
					</Button>
				</DropDown>
			{/if}

			<Button secondary hideOnMobile>
				{#snippet icon()}
					<Gift />
				{/snippet}
				{$t("labels.gift")}
			</Button>
			<Button hideOnDesktop>
				{#snippet icon()}
					<Gift />
				{/snippet}
			</Button>

			<DropDown>
				{#snippet dropdown()}
					<Button big href="/cosmetics">
						{#snippet icon()}
							<PaintBrush />
						{/snippet}
						Your Cosmetics
					</Button>
					{#if subInfo.activePeriod && !subInfo.activePeriod.giftedBy}
						{#if subInfo.activePeriod.subscription.state === SubscriptionState.Active}
							<Button big style="color: var(--danger)" onclick={cancelSubscription}>
								{#snippet icon()}
									{#if cancelSubLoading}
										<Spinner />
									{:else}
										<Warning />
									{/if}
								{/snippet}
								Cancel Subscription
							</Button>
						{:else if subInfo.activePeriod.subscription.state === SubscriptionState.CancelAtEnd}
							<Button big style="color: var(--store)" onclick={renewSubscription}>
								{#snippet icon()}
									{#if renewSubLoading}
										<Spinner />
									{:else}
										<Star />
									{/if}
								{/snippet}
								Reactivate Subscription
							</Button>
						{/if}
					{/if}
				{/snippet}
				<Button secondary>
					{#snippet icon()}
						<DotsThreeVertical />
					{/snippet}
				</Button>
			</DropDown>
		</div>
	{/snippet}
	<SubInfo data={subInfo} />
</StoreSection>

<style lang="scss">
	.buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
