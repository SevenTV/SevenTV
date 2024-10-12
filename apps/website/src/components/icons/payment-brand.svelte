<script lang="ts">
	import Fa from "svelte-fa";
	import { faCreditCard } from "@fortawesome/free-regular-svg-icons";
	import {
		type IconDefinition,
		faAlipay,
		faIdeal,
		faCcPaypal,
		faCcAmex,
		faCcDinersClub,
		faCcDiscover,
		faCcJcb,
		faCcMastercard,
		faCcVisa,
	} from "@fortawesome/free-brands-svg-icons";

	const icons: { [key: string]: IconDefinition } = {
		alipay: faAlipay,
		ideal: faIdeal,
		paypal: faCcPaypal,

		amex: faCcAmex,
		diners: faCcDinersClub,
		discover: faCcDiscover,
		jcb: faCcJcb,
		mastercard: faCcMastercard,
		visa: faCcVisa,
	};

	const names: { [key: string]: string } = {
		alipay: "Alipay",
		ideal: "iDEAL",
		paypal: "PayPal",

		amex: "American Express",
		diners: "Diners Club",
		discover: "Discover",
		jcb: "JCB",
		mastercard: "Mastercard",
		visa: "Visa",
	};

	export let type: string;
</script>

<div class="payment-method" title={names[type]}>
	<Fa icon={icons[type] ?? faCreditCard} size="2x" fw />
	{#if names[type]}
		{names[type]}
	{/if}
</div>

<style lang="scss">
	.payment-method {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}
</style>
