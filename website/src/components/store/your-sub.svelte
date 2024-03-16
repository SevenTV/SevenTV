<script lang="ts">
	import Button from "../input/button.svelte";
	import SubInfo from "../sub-info.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretDown, DotsThreeVertical, Gift, Star } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	export let subbed = false;
</script>

<StoreSection title={$t("common.your_subscription")}>
	<div class="buttons" slot="header">
		<Button
			secondary
			hideOnMobile
			on:click={() => (subbed = true)}
			style={subbed && "color: var(--subscriber)"}
		>
			<Star weight={subbed ? "fill" : "bold"} slot="icon" />
			<span>
				{subbed
					? $t("pages.store.subscription.subscribed")
					: $t("pages.store.subscription.subscribe")}
			</span>
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
			{$t("labels.gift")}
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
