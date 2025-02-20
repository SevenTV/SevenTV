<script lang="ts">
	import { type SubscriptionProductVariant } from "$/gql/graphql";
	import { ArrowRight } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { priceFormat } from "$/lib/utils";
	import { purchasePickems } from "$/lib/pickems";
	import { isSubscribed } from "$/lib/auth";
	let { variant, title }: { variant?: SubscriptionProductVariant; title: string } = $props();

	let price = $derived(priceFormat("EUR").format((499 + (variant?.price.amount ?? 0)) / 100));
	let discounted = $derived(priceFormat("EUR").format((300 + (variant?.price.amount ?? 0)) / 100));
	let recurring = $derived(priceFormat("EUR").format((variant?.price.amount ?? 0) / 100));

	let disabled = $derived(variant && isSubscribed);
</script>

<div class="purchase-option" class:disabled>
	<h3>{title}</h3>
	<div class="line" class:strike={variant}>
		<span>Regular: </span>
		<span>{price}</span>
	</div>
	<div class="line">
		{#if variant}
			<span>Bundle: </span>
			<span>{discounted}</span>
		{:else}
			<wbr />
		{/if}
	</div>
	<Button primary={!disabled} disabled={!!disabled} onclick={() => purchasePickems(variant?.id)}>
		{#snippet iconRight()}
			<ArrowRight />
		{/snippet}
		{#if !disabled}
			Buy Now
		{:else}
			Already Subscribed
		{/if}
	</Button>
	<small>
		<small>
			{#if variant}
				{price} today, then {recurring} a {variant.kind.toLowerCase()}
			{:else}
				no subscription
			{/if}
		</small>
	</small>
</div>

<style lang="scss">
	.purchase-option {
		flex-grow: 1;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		justify-content: space-between;
		border: 2px solid black;
		border-radius: 1rem;
		padding: 1rem;

		&.disabled {
			opacity: 0.75;
		}

		.line {
			display: flex;
			justify-content: space-between;

			&.strike {
				text-decoration: line-through;
			}
		}

		:global(button) {
			margin-top: 1rem;
		}

		small {
			text-align: center;
		}
	}
</style>
