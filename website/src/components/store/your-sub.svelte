<script lang="ts">
	import Button from "../input/button.svelte";
	import SubInfo from "../sub-info.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretDown, DotsThreeVertical, Gift, Star } from "phosphor-svelte";

	export let subbed = false;
</script>

<StoreSection title="Your Subscription">
	<div class="buttons" slot="header">
		<Button
			secondary
			hideOnMobile
			on:click={() => (subbed = true)}
			style={subbed && "color: var(--subscriber)"}
		>
			<Star weight={subbed ? "fill" : "bold"} slot="icon" />
			<span>{subbed ? "Subscribed" : "Subscribe"}</span>
			<CaretDown slot="icon-right" />
		</Button>
		<Button
			secondary
			hideOnDesktop
			on:click={() => (subbed = true)}
			style={subbed && "color: var(--subscriber)"}
		>
			<Star weight={subbed ? "fill" : "bold"} slot="icon" />
			<CaretDown slot="icon-right" />
		</Button>

		<Button secondary hideOnMobile>
			<Gift slot="icon" />
			Gift
		</Button>
		<Button hideOnDesktop>
			<Gift slot="icon" />
		</Button>

		<Button secondary>
			<DotsThreeVertical slot="icon" />
		</Button>
	</div>
	<SubInfo />
</StoreSection>

<style lang="scss">
	.buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
