<script lang="ts">
	import Button from "../button.svelte";
	import StoreSection from "./store-section.svelte";
	import {
		CaretDown,
		Clock,
		DotsThreeVertical,
		Gift,
		Hourglass,
		Sparkle,
		Star,
	} from "phosphor-svelte";

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
	<div class="sub-grid">
		<span class="key">Plan</span>
		<span class="value">
			<Sparkle />
			<span>Free</span>
		</span>

		<span class="key">Gifted By</span>
		<span class="value">
			<Gift />
			<span>N/A</span>
		</span>

		<span class="key">Sub Ends</span>
		<span class="value">
			<Hourglass />
			<span>N/A</span>
		</span>

		<span class="key">Previously Subscribed</span>
		<span class="value">
			<Clock />
			<span>0 days</span>
		</span>
	</div>
</StoreSection>

<style lang="scss">
	.buttons {
		display: flex;
		gap: 0.5rem;
	}

	.sub-grid {
		padding: 0.75rem;

		display: grid;
		grid-template-rows: repeat(2, auto);
		grid-auto-flow: column;
		row-gap: 0.5rem;
		column-gap: 2rem;

		.key {
			color: var(--text-light);
			font-size: 0.75rem;
			font-weight: 500;
		}

		.value {
			display: flex;
			gap: 0.5rem;
			align-items: center;

			color: var(--subscriber);

			span {
				color: var(--text);
				font-size: 0.875rem;
				font-weight: 600;
			}
		}
	}

	@media screen and (max-width: 960px) {
		.sub-grid {
			grid-template-rows: repeat(4, auto);
		}
	}
</style>
