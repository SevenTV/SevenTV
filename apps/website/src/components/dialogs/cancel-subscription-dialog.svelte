<script lang="ts">
	import { graphql } from "$/gql";
	import type { SubscriptionInfo } from "$/gql/graphql";
	import { user } from "$/lib/auth";
	import { gqlClient } from "$/lib/gql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import moment from "moment";
	import Date from "../date.svelte";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";

	interface Props {
		mode: DialogMode;
		subInfo: SubscriptionInfo;
	}

	let { mode = $bindable("hidden"), subInfo = $bindable() }: Props = $props();

	let loading = $state(false);

	async function cancelSubscription() {
		if (!$user) {
			return;
		}

		loading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation CancelSubscription($userId: Id!, $productId: Id!) {
						billing(userId: $userId) {
							cancelSubscription(productId: $productId) {
								totalDays
								activePeriod {
									subscriptionProductVariant {
										kind
									}
									subscription {
										state
									}
									end
									autoRenew
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
			subInfo = res.data.billing.cancelSubscription as SubscriptionInfo;
		}

		loading = false;
		mode = "hidden";
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>{subInfo.activePeriod?.giftedBy ? "End your subscription" : "Cancel your subscription"}</h1>
		<hr />
		{#if subInfo.activePeriod?.giftedBy}
			<p>
				{$t("dialogs.subscription.gifted_by")} {subInfo.activePeriod.giftedBy.mainConnection
					?.platformDisplayName}. {$t("dialogs.subscription.cancel_notice")}
			</p>
		{:else}
			<p>
				{$t("dialogs.subscription.cancel_confirmation")} (<Date
					date={moment(subInfo.activePeriod?.end)}
				/>)
			</p>
		{/if}
		<div class="buttons">
			<Button primary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>

			{#snippet spinnerIcon()}
				<Spinner />
			{/snippet}

			<Button
				style="color: var(--danger)"
				icon={loading ? spinnerIcon : undefined}
				disabled={loading}
				onclick={cancelSubscription}
				submit
			>
				{$t("dialogs.subscription.confirm")}
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

	.buttons {
		margin-top: auto;
		margin-left: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
