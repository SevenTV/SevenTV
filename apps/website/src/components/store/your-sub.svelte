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
		type SubscriptionProductVariant,
	} from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import { user } from "$/lib/auth";
	import Spinner from "../spinner.svelte";
	import type { DialogMode } from "../dialogs/dialog.svelte";
	import CancelSubscriptionDialog from "../dialogs/cancel-subscription-dialog.svelte";
	import { variantName } from "$/lib/utils";
	import GiftSubscriptionDialog from "../dialogs/gift-subscription-dialog.svelte";

	interface Props {
		subInfo: SubscriptionInfo;
		product: SubscriptionProduct;
	}

	let { subInfo = $bindable(), product }: Props = $props();

	let subscribeLoading = $state<string>();

	async function subscribe(variantId: string) {
		if (!$user) {
			return;
		}

		subscribeLoading = variantId;

		const res = await gqlClient().mutation(
			graphql(`
				mutation Subscribe($userId: Id!, $variantId: ProductId!) {
					billing(userId: $userId) {
						subscribe(variantId: $variantId) {
							checkoutUrl
						}
					}
				}
			`),
			{ userId: $user.id, variantId },
		);

		if (res.data) {
			window.location.href = res.data.billing.subscribe.checkoutUrl;
		}

		subscribeLoading = undefined;
	}

	let reactivateSubLoading = $state(false);

	async function reactivateSubscription() {
		if (!$user) {
			return;
		}

		reactivateSubLoading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation ReactivateSubscription($userId: Id!, $productId: Id!) {
						billing(userId: $userId) {
							reactivateSubscription(productId: $productId) {
								totalDays
								activePeriod {
									subscriptionProductVariant {
										kind
									}
									subscription {
										state
									}
									end
									giftedBy {
										id
										mainConnection {
											platformDisplayName
										}
										style {
											activePaint {
												id
												name
												data {
													layers {
														id
														ty {
															__typename
															... on PaintLayerTypeSingleColor {
																color {
																	hex
																}
															}
															... on PaintLayerTypeLinearGradient {
																angle
																repeating
																stops {
																	at
																	color {
																		hex
																	}
																}
															}
															... on PaintLayerTypeRadialGradient {
																repeating
																stops {
																	at
																	color {
																		hex
																	}
																}
																shape
															}
															... on PaintLayerTypeImage {
																images {
																	url
																	mime
																	size
																	scale
																	width
																	height
																	frameCount
																}
															}
														}
														opacity
													}
													shadows {
														color {
															hex
														}
														offsetX
														offsetY
														blur
													}
												}
											}
										}
										highestRoleColor {
											hex
										}
									}
								}
							}
						}
					}
				`),
				{
					userId: $user.id,
					productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
				},
			)
			.toPromise();

		if (res.data) {
			subInfo = res.data.billing.reactivateSubscription as SubscriptionInfo;
		}

		reactivateSubLoading = false;
	}

	let cancelSubDialog: DialogMode = $state("hidden");
	let giftSubDialog: DialogMode = $state("hidden");
	let giftSubVariant = $state<SubscriptionProductVariant>();

	function showGiftDialog(variant: SubscriptionProductVariant) {
		giftSubVariant = variant;
		giftSubDialog = "shown";
	}
</script>

<CancelSubscriptionDialog bind:mode={cancelSubDialog} bind:subInfo />
{#if giftSubVariant}
	<GiftSubscriptionDialog bind:mode={giftSubDialog} variant={giftSubVariant} />
{/if}
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
							<Button
								big
								onclick={() => subscribe(variant.id)}
								disabled={subscribeLoading !== undefined}
								style="width: 100%"
							>
								{#snippet icon()}
									{#if subscribeLoading === variant.id}
										<Spinner />
									{:else}
										<Star />
									{/if}
								{/snippet}
								{variantName(variant)}
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

			<DropDown>
				{#snippet dropdown()}
					{#each product.variants as variant}
						<Button big onclick={() => showGiftDialog(variant)} style="width: 100%">
							{#snippet icon()}
								<Star />
							{/snippet}
							{variantName(variant)}
						</Button>
					{/each}
				{/snippet}
				<Button secondary hideOnMobile>
					{#snippet icon()}
						<Gift />
					{/snippet}
					{$t("labels.gift")}
					{#snippet iconRight()}
						<CaretDown />
					{/snippet}
				</Button>
				<Button hideOnDesktop>
					{#snippet icon()}
						<Gift />
					{/snippet}
				</Button>
			</DropDown>

			<DropDown>
				{#snippet dropdown()}
					<Button big href="/cosmetics">
						{#snippet icon()}
							<PaintBrush />
						{/snippet}
						Your Cosmetics
					</Button>
					{#if subInfo.activePeriod}
						{#if subInfo.activePeriod.subscription.state === SubscriptionState.Active}
							<Button big style="color: var(--danger)" onclick={() => (cancelSubDialog = "shown")}>
								{#snippet icon()}
									<Warning />
								{/snippet}
								Cancel Subscription
							</Button>
						{:else if subInfo.activePeriod.subscription.state === SubscriptionState.CancelAtEnd && !subInfo.activePeriod.giftedBy}
							<Button
								big
								style="color: var(--store)"
								onclick={reactivateSubscription}
								disabled={reactivateSubLoading}
							>
								{#snippet icon()}
									{#if reactivateSubLoading}
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
