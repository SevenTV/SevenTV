<script lang="ts">
	import { graphql } from "$/gql";
	import { type PickemsStoreDataQuery } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import { ArrowSquareOut, Info } from "phosphor-svelte";
	import type { PageData } from "./$types";
	import StoreSection from "$/components/store/store-section.svelte";
	import PickemsPurchaseButton from "$/components/pickems/purchase-button.svelte";
	import { user } from "$/lib/auth";
	import Button from "$/components/input/button.svelte";
	import StorePickemsBanner from "$/components/store/store-pickems-banner.svelte";
	import PickemsStreamers from "$/components/pickems/pickems-streamers.svelte";
	import PickemsSchedule from "$/components/pickems/pickems-schedule.svelte";

	let { data }: { data: PageData } = $props();

	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);

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

	$effect(() => {
		queryStore().then((res) => {
			storeData = res;
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
			<StoreSection title="Go to app.pickems.tv to place your Pick'ems!">
				{#snippet header()}
					<Button primary href="https://app.pickems.tv">
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
			<PickemsStreamers {hasPass} />
			<div class="container" id="PickemsPricing">
				<h1 class="title">7TV Pick’ems Pass</h1>
				<p class="description">
					<span class="dashed-line"></span>
				</p>
				<div class="buttons">
					{#if !hasPass}
						<PickemsPurchaseButton title="PICKEMS PASS ONLY" />
						{#each storeData?.products.subscriptionProduct?.variants ?? [] as variant}
							<PickemsPurchaseButton title={`PASS + ${variant.kind} SUB`} {variant} />
						{/each}
					{:else}
						<PickemsPurchaseButton title="GIFT PASS" gift />
					{/if}
				</div>
			</div>
			<PickemsSchedule />
		</div>
	</div>
</div>

<style>
	.grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		flex-wrap: wrap;
		min-width: 100%;
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
		border: 1px solid rgb(255 255 255 / 12%);
		border-radius: 0.5rem;
		text-align: center;

		.title {
			text-align: center;
			font-family: Inter, serif;
			letter-spacing: -1.28px;
			background: linear-gradient(0deg, #fff -24.36%, rgba(255, 255, 255, 0.61) 116.67%);
			background-clip: text;
			-webkit-background-clip: text;
			-webkit-text-fill-color: transparent;
			padding-left: 0;
			font-size: 2.5rem;
			margin: 0 4rem 4rem;
			font-weight: 700;
		}

		.description {
			font-size: 1rem;
			color: white;
			margin-bottom: 1rem;
			position: relative;

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

		@media screen and (max-width: 1200px) {
			.buttons {
				flex-direction: column !important;
				align-items: center;
			}
			.title {
				font-size: 2rem;
			}
		}
	}

	@media screen and (max-width: 1600px) {
		.container {
			padding: 3rem 4rem !important;
		}
	}

	@media screen and (max-width: 1400px) {
		.container {
			padding: 3rem 3rem !important;
		}
		.title {
			margin: unset !important;
		}
	}
</style>
