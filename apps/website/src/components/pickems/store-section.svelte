<script lang="ts">
	import { t } from "svelte-i18n";
	import type { SubscriptionInfo, SubscriptionProduct } from "$/gql/graphql";
	import { ArrowSquareOut, CaretDown, Spinner, Ticket } from "phosphor-svelte";
	import StoreSection from "../store/store-section.svelte";
	import Button from "../input/button.svelte";
	import DropDown from "../drop-down.svelte";
	import { priceFormat, variantName } from "$/lib/utils";
	import { user } from "$/lib/auth";
	import { purchasePickems } from "$/lib/pickems";

	interface Props {
		subInfo?: SubscriptionInfo;
		product?: SubscriptionProduct;
	}

	let purchaseLoading = $state<string>();
	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);

	async function purchase(variantId?: string) {
		purchaseLoading = variantId ?? "pass";
		await purchasePickems(variantId);
		purchaseLoading = undefined;
	}

	let { subInfo, product }: Props = $props();
</script>

<StoreSection title="Pick'ems Pass">
	{#snippet header()}
		<div class="buttons">
			{#if hasPass}
				<Button href="https://app.pickems.tv" secondary>
					{#snippet icon()}
						<ArrowSquareOut />
					{/snippet}
					<span> {$t("pages.store.events.cs2.pickems.place")} </span>
				</Button>
			{:else if subInfo?.activePeriod}
				<Button onclick={() => purchase()} secondary style="color: var(--store)">
					{#snippet icon()}
						<Ticket />
					{/snippet}
					<span> {$t("pages.store.events.cs2.pass.get")} </span>
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
								<span> {$t("pages.store.events.cs2.pass.title")} </span>
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
										{$t("pages.store.events.cs2.pass.bundle")}
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
