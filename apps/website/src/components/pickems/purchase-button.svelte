<script lang="ts">
	import { type SubscriptionProductVariant } from "$/gql/graphql";
	import { ArrowRight } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { priceFormat } from "$/lib/utils";
	import { purchasePickems } from "$/lib/pickems";
	// import { isSubscribed } from "$/lib/auth";

	let {
		variant,
		title,
		myStoreData,
	}: { variant?: SubscriptionProductVariant; title: string; myStoreData?: boolean } = $props();

	let price = $derived(priceFormat("EUR").format((499 + (variant?.price.amount ?? 0)) / 100 - 1.49));
	// let discounted = $derived(priceFormat("EUR").format((300 + (variant?.price.amount ?? 0)) / 100));
	let recurring = $derived(priceFormat("EUR").format((variant?.price.amount ?? 0) / 100));
	let disabled = $derived(myStoreData);
</script>

<div class="purchase-option" class:disabled id="PickemsPurchaseButton">
	<h3>{title}</h3>
	<hr class="hrDialog" />
	<div class="line">
		{#if variant}
			<span>{price}</span>
		{:else}
		<span>€4.99</span>
		{/if}
	</div>
	<Button
		primary
		style="width: 70%; justify-content: space-between; box-shadow: rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset, rgba(0, 0, 0, 0.2) 0px 6px 4px; filter: drop-shadow(0 0 10px rgba(255, 255, 255, 0.5));"
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
			{price} today, then {recurring} {variant.kind.toLowerCase()}
		{:else}
			Billed as one-time purchase
		{/if}
	</small>
</div>

<style lang="scss">

	span {
		font-size: 1.8rem;
		font-weight: normal;
	}
	.purchase-option {
		flex-grow: 1;
		display: flex;
		flex-direction: column;
		gap: 1.75rem;
		align-items: center;
		border-radius: 0.8rem;
		text-align: center;
		background: rgb(255 255 255 / 10%);
		border: 1px solid #ffffff30;
		box-shadow: inset 0 -4px 9px -3px #ffffff52;
		padding: 1.2rem 0.8rem;
		justify-content: space-between;
		width: 100% !important;
		font-weight: 600;
		font-size: 1.1rem;

		&.disabled {
			opacity: 0.6;
		}

		.hrDialog {
			background: rgba(255, 255, 255, 0.07);
			width: 100%;
			margin: 0;
			border-color: #ffffff;
			opacity: 0.2;
		}

		h3 {
			font-family: Inter, sans-serif;
			font-size: 1.5rem;
			font-style: normal;
			font-weight: 600;
			line-height: 120%; /* 67.2px */
			background: linear-gradient(0deg, #fff -24.36%, rgba(255, 255, 255, 0.61) 116.67%);
			background-clip: text;
			-webkit-background-clip: text;
			-webkit-text-fill-color: transparent;
			text-align: center;
			width: 100%;
			padding-left: 1rem;
		}

		.line {
			display: flex;
			justify-content: center;
			width: 100%;

			//&.strike {
			//	text-decoration: line-through;
			//	color: #cfcfcf;
			//	span {
			//		color: white;
			//		opacity: 0.4;
			//	}
			//}
		}

		//button {
		//	padding: 1.2rem 0.8rem;
		//	justify-content: space-between;
		//	width: 100% !important;
		//	font-weight: 600;
		//	font-size: 1.1rem;
		//	box-shadow:
		//		rgba(127, 127, 127, 0.25) 2px 5px 4px 0px inset,
		//		rgba(0, 0, 0, 0.2) 0px 6px 4px !important;
		//	filter: drop-shadow(rgba(255, 255, 255, 0.3) 0px 0px 10px) !important;
		//}

		small {
			font-size: 0.85rem;
			color: gray;
		}
	}
</style>
