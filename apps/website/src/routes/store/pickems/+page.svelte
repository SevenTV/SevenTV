<script lang="ts">
	import { graphql } from "$/gql";
	import {
		type Badge,
		type MyStoreDataQuery,
		type Paint,
		type PickemsStoreDataQuery,
	} from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import { ArrowSquareOut, Info, Minus } from "phosphor-svelte";
	import type { PageData } from "./$types";
	import StoreSection from "$/components/store/store-section.svelte";
	import { queryPickemsCosmetics } from "$/lib/pickems";
	import PickemsPurchaseButton from "$/components/pickems/purchase-button.svelte";
	import { user } from "$/lib/auth";
	import Button from "$/components/input/button.svelte";
	import StorePickemsBanner from "$/components/store/store-pickems-banner.svelte";
	import PickemsStreamers from "$/components/pickems/pickems-streamers.svelte";
	import PickemsSchedule from "$/components/pickems/pickems-schedule.svelte";

	let { data }: { data: PageData } = $props();

	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);

	async function queryMyStore(userId: string) {
		let res = await gqlClient()
			.query(
				graphql(`
					query PickemsMyStoreData($userId: Id!, $productId: Id!) {
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
										endDate
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
						}
					}
				`),
				{
					userId,
					productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
				},
			)
			.toPromise();

		return res.data;
	}

	async function queryStore() {
		let res = await gqlClient()
			.query(
				graphql(`
					query PickemsStoreData($productId: Id!) {
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
					}
				`),
				{
					productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID,
				},
			)
			.toPromise();

		return res.data;
	}

	let storeData = $state<PickemsStoreDataQuery>();
	let myStoreData = $state<MyStoreDataQuery>();
	let badges = $state<Badge[]>([]);
	let paints = $state<Paint[]>([]);

	$effect(() => {
		queryStore().then((res) => {
			storeData = res;
		});

		if ($user) {
			queryMyStore($user.id).then((res) => {
				myStoreData = res;
			});
		}

		queryPickemsCosmetics().then((cosmetics) => {
			badges = cosmetics.badges;
			paints = cosmetics.paints;
		});
	});
</script>

<svelte:head>
	<title>Pickems</title>
</svelte:head>

<StorePickemsBanner title="7TV x CSMONEY" title2="Streamer Invitational" />

<div class="grid">
	<!-- <img alt="Pickems Banner" class="banner-image" src={pickemsHeaderImage} /> -->
	{#if data.success}
		<div class="bar">
			<Info />
			Pickems pass successfully purchased
		</div>
	{/if}
	{#if hasPass}
		<div class="bar">
			<StoreSection title="Go to pickems.tv to place your Pick'ems!">
				{#snippet header()}
					<Button primary href="https://pickems.tv">
						{#snippet iconRight()}
							<ArrowSquareOut />
						{/snippet}
						Place Pick'ems
					</Button>
				{/snippet}
			</StoreSection>
		</div>
	{/if}
	<div class="top-grid">
		<div class="subgrid">
			{#if !hasPass}
				<PickemsStreamers hasPass={false} />
			{:else}
				<PickemsStreamers hasPass={true} />
			{/if}
			{#if !hasPass}
				<div class="container">
					<h1 class="title">7TV Pick’ems Pass</h1>
					<p class="description">
						<span class="dashed-line"></span>
					</p>
					<div class="buttons">
						<PickemsPurchaseButton title="PICKEMS PASS ONLY" />
						{#each storeData?.products.subscriptionProduct?.variants ?? [] as variant}
							<PickemsPurchaseButton
								title={`PASS + ${variant.kind} SUB`}
								{variant}
								myStoreData={!!myStoreData?.users.user?.billing.subscriptionInfo.activePeriod}
							/>
						{/each}
					</div>
				</div>
			{/if}
			<PickemsSchedule hasPass />
		</div>
	</div>
</div>

<style lang="scss">
	.grid {
		min-width: 100% !important;
	}
	.banner-image {
		object-fit: contain;
		width: 100%;
		margin-top: -20%;
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

	.bar {
		background-color: var(--bg-light);
		color: var(--text);
		padding: 0.5rem;
		border-radius: 0.25rem;
		border: 1px solid var(--store);

		display: flex;
		justify-content: center;
		align-items: center;
		gap: 0.5rem;
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

	.container {
		padding: 3rem 8rem;
		background: radial-gradient(304.85% 93.65% at 50% 83.99%, #420097 0%, rgba(45, 0, 104, 0) 100%);
		border-radius: 12px;
		border: 1px solid rgb(255 255 255 / 12%);
		border-radius: 0.5rem;
		text-align: center;

		.title {
			text-align: center;
			font-family: Inter;
			letter-spacing: -1.28px;
			background: linear-gradient(0deg, #fff -24.36%, rgba(255, 255, 255, 0.61) 116.67%);
			background-clip: text;
			-webkit-background-clip: text;
			margin: 4rem;
			-webkit-text-fill-color: transparent;
			padding-left: 0;
			font-size: 2.5rem;
			margin-top: 0;
			font-weight: 700;
		}

		.description {
			font-size: 1rem;
			color: white;
			margin-bottom: 1rem;
			position: relative;

			.dialog {
				position: absolute;
				top: -1rem;
				font-size: 1rem;
				color: white;
			}

			.left {
				left: 15%;
			}

			.middle {
				left: 50%;
				transform: translateX(-50%);
			}

			.right {
				left: 82%;
			}

			.dashed-line {
				display: block;
				width: 100%;
				border-top: 2px dashed #3b393987;
				margin-top: 0.5rem;
			}
		}

		.buttons {
			display: flex;
			flex-direction: row;
			gap: 1rem;
			justify-content: center;
		}
	}
</style>
