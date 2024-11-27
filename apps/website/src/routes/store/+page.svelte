<script lang="ts">
	import Spinner from "$/components/spinner.svelte";
	import BadgeProgressComponent from "$/components/store/badge-progress.svelte";
	import Banner from "$/components/store/banner.svelte";
	import Benefits from "$/components/store/benefits.svelte";
	import MonthlyPaints from "$/components/store/monthly-paints.svelte";
	import PersonalEmotes from "$/components/store/personal-emotes.svelte";
	import YourSub from "$/components/store/your-sub.svelte";
	import { graphql } from "$/gql";
	import {
		type BadgeProgress,
		type Paint,
		type StoreDataQuery,
		type SubscriptionInfo,
		type SubscriptionProduct,
	} from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { PaintBrush, Seal, Smiley, UserCircle } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { user } from "$/lib/auth";
	import SignInDialog from "$/components/dialogs/sign-in-dialog.svelte";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";

	async function queryStore(userId: string) {
		let res = await gqlClient().query(
			graphql(`
				query StoreData($userId: Id!, $productId: Id!) {
					products {
						subscriptionProduct(id: $productId) {
							id
							name
							variants {
								id
								price {
									amount
									currency
								}
								kind
							}
						}
					}
					users {
						user(id: $userId) {
							billing(productId: $productId) {
								badgeProgress {
									currentBadge {
										id
										name
										description
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
									nextBadge {
										badge {
											id
											name
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
										percentage
										daysLeft
									}
								}
								subscriptionInfo {
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
					}
					store {
						monthlyPaints {
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
				}
			`),
			{
				userId,
				productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
			},
		);

		return res.data;
	}

	let data = $state<StoreDataQuery>();

	$effect(() => {
		if ($user) {
			queryStore($user.id).then((res) => {
				data = res;
			});
		}
	});
</script>

<svelte:head>
	<title>{$t("common.subscriptions", { values: { count: 1 } })} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#snippet banner(subbed: boolean)}
	<Banner
		title={subbed
			? $t("pages.store.subscription.banner_title_subbed")
			: $t("pages.store.subscription.banner_title_unsubbed")}
		subtitle={subbed
			? $t("pages.store.subscription.banner_subtitle_subbed")
			: $t("pages.store.subscription.banner_subtitle_unsubbed")}
	>
		<div class="banner-icons hide-on-mobile">
			<PaintBrush size="1.8rem" />
			<UserCircle size="1.8rem" />
			<Seal size="1.8rem" />
			<!-- <Ticket size="1.8rem" /> -->
			<Smiley size="1.8rem" />
		</div>
	</Banner>
{/snippet}

{#await data}
	{@render banner(false)}
{:then data}
	{@render banner(!!data?.users.user?.billing.subscriptionInfo.activePeriod)}
{/await}
<!-- All things called grid here aren't actually css grids -->
<div class="grid">
	{#await data}
		<Benefits />
	{:then data}
		{#if !data?.users.user?.billing.subscriptionInfo.activePeriod}
			<Benefits />
		{/if}
	{/await}
	<div class="top-grid">
		{#await data}
			<div class="spinner-container">
				<Spinner />
			</div>
		{:then data}
			{#if data}
				<div class="subgrid">
					{#if data.users.user && data.products.subscriptionProduct}
						<YourSub
							bind:subInfo={data.users.user.billing.subscriptionInfo as SubscriptionInfo}
							product={data.products.subscriptionProduct as SubscriptionProduct}
						/>
						<BadgeProgressComponent
							progress={data.users.user.billing.badgeProgress as BadgeProgress}
						/>
					{/if}
				</div>
				<MonthlyPaints paints={data.store.monthlyPaints as Paint[]} />
			{/if}
		{/await}
	</div>
	<PersonalEmotes />
	<!-- <div class="three-grid">
		<EmoteRaffle />
		<PersonalEmotes />
		<TopGifters />
	</div> -->
	{#await data then data}
		{#if data?.users.user?.billing.subscriptionInfo.activePeriod}
			<Benefits />
		{/if}
	{/await}
	{#if $user === null}
		<SignInDialog mode="shown-without-close" />
	{/if}
	<!-- <Faq /> -->
</div>

<style lang="scss">
	.banner-icons {
		padding: 0 2.75rem;

		display: flex;
		gap: 3.5rem;
		flex-wrap: wrap;
		align-items: center;
		justify-content: center;
	}

	.grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		flex-wrap: wrap;

		max-width: 70rem;
		margin-top: 1rem;
		margin-inline: auto;
	}

	.top-grid {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;

		& > .subgrid {
			flex-grow: 1;

			display: flex;
			flex-direction: column;
			gap: 1rem;
			flex-wrap: wrap;
		}
	}

	.spinner-container {
		margin: 0 auto;
	}

	// .three-grid {
	// 	display: flex;
	// 	gap: 1rem;
	// 	flex-wrap: wrap;
	// }
</style>
