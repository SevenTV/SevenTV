<script lang="ts">
	import { graphql } from "$/gql";
	import { type Badge, type Paint, type PickemsStoreDataQuery } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import { ArrowSquareOut, Info } from "phosphor-svelte";
	import type { PageData } from "./$types";
	import StoreSection from "$/components/store/store-section.svelte";
	import { queryPickemsCosmetics } from "$/lib/pickems";
	import PickemsBadges from "$/components/pickems/pickems-badges.svelte";
	import PickemsPaints from "$/components/pickems/pickems-paints.svelte";
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
	let badges = $state<Badge[]>([]);
	let paints = $state<Paint[]>([]);

	$effect(() => {
		queryStore().then((res) => {
			storeData = res;
		});

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
			<PickemsStreamers />
			{#if !hasPass}
				<StoreSection>
					<div class="top-grid">
						<PickemsPurchaseButton title="PICKEMS PASS ONLY" />
						{#each storeData?.products.subscriptionProduct?.variants ?? [] as variant}
							<PickemsPurchaseButton title={`PASS + ${variant.kind} SUB`} {variant} />
						{/each}
					</div>
				</StoreSection>
			{/if}
			<PickemsSchedule />
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
</style>
