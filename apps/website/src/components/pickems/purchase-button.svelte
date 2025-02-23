<script lang="ts">
	import { type SubscriptionProductVariant } from "$/gql/graphql";
	import { ArrowRight, Password } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { priceFormat } from "$/lib/utils";
	import { purchasePickems } from "$/lib/pickems";
	import { isSubscribed } from "$/lib/auth";

	let {
		variant,
		title,
		myStoreData,
	}: { variant?: SubscriptionProductVariant; title: string; myStoreData?: boolean } = $props();

	let price = $derived(priceFormat("EUR").format((499 + (variant?.price.amount ?? 0)) / 100));
	let discounted = $derived(priceFormat("EUR").format((300 + (variant?.price.amount ?? 0)) / 100));
	let recurring = $derived(priceFormat("EUR").format((variant?.price.amount ?? 0) / 100));
	let disabled = $derived(myStoreData);
</script>

<div class="purchase-option" class:disabled id="PickemsPurchaseButton">
	<h3>{title}</h3>
	<hr class="hrDialog" />
	<div class="line strike" class:strike={variant}>
		<span>Regular: </span>
		<span>{price}</span>
	</div>
	<div class="line">
		{#if variant}
			<span>Bundle: </span>
			<span>{discounted}</span>
		{/if}
	</div>
	<Button
		primary
		style="width: 70%; box-shadow: rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset, 0 4px 8px rgba(0, 0, 0, 0.2); filter: drop-shadow(0 0 10px rgba(255, 255, 255, 0.5));"
		disabled={myStoreData}
		onclick={() => purchasePickems(variant?.id)}
	>
		{#snippet iconRight()}
			<ArrowRight />
		{/snippet}
		{#if !myStoreData}
			{#if variant}
				Purchase Bundle
			{:else}
				Purchase Pass
			{/if}
		{:else}
			Already Subscribed
		{/if}
	</Button>
	<small>
		{#if variant}
			{price} today, then {recurring} monthly
		{:else}
			Billed as one-time purchase
		{/if}
	</small>
</div>

<style lang="scss">
	.purchase-option {
		flex-grow: 1;
		display: flex;
		flex-direction: column;
		gap: 1.75rem;
		align-items: center;
		justify-content: space-between;
		border-radius: 1rem;
		padding: 3.5rem;
		text-align: center;
		background: rgba(0, 0, 0, 0.288);
		&.disabled {
			opacity: 0.6;
		}

		.hrDialog {
			background: rgba(255, 255, 255, 0.07);
			width: 100%;
			margin: 0;
		}

		h3 {
			font-family: Inter;
			font-size: 56px;
			font-style: normal;
			font-weight: 600;
			line-height: 120%; /* 67.2px */
			background: linear-gradient(0deg, #fff -24.36%, rgba(255, 255, 255, 0.61) 116.67%);
			background-clip: text;
			-webkit-background-clip: text;
			-webkit-text-fill-color: transparent;
			text-align: left;
			width: 100%;
			padding-left: 1rem; /* Adjust this value as needed */
		}

		.line {
			display: flex;
			justify-content: space-between;
			width: 75%;

			&.strike {
				text-decoration: line-through;
				color: gray;
			}
		}

		button {
			margin-top: 1rem;
			width: 70%;
		}

		small {
			font-size: 0.85rem;
			color: gray;
		}
	}
</style>
