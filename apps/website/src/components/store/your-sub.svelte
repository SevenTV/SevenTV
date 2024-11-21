<script lang="ts">
	import Button from "../input/button.svelte";
	import SubInfo from "../sub-info.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretDown, DotsThreeVertical, Gift, PaintBrush, Star, Warning } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	let { subbed = $bindable(false) }: { subbed?: boolean } = $props();
	import DropDown from "../drop-down.svelte";
</script>

<StoreSection title={$t("common.your_subscription")}>
	{#snippet header()}
		<div class="buttons">
			<Button
				secondary
				hideOnMobile
				onclick={() => (subbed = true)}
				style={subbed ? "color: var(--store)" : undefined}
			>
				{#snippet icon()}
					<Star weight={subbed ? "fill" : "bold"} />
				{/snippet}
				<span>
					{subbed
						? $t("pages.store.subscription.subscribed")
						: $t("pages.store.subscription.subscribe")}
				</span>
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>
			<Button
				secondary
				hideOnDesktop
				onclick={() => (subbed = true)}
				style={subbed ? "color: var(--store)" : undefined}
			>
				{#snippet icon()}
					<Star weight={subbed ? "fill" : "bold"} />
				{/snippet}
				{#snippet iconRight()}
					<CaretDown />
				{/snippet}
			</Button>

			<Button secondary hideOnMobile>
				{#snippet icon()}
					<Gift />
				{/snippet}
				{$t("labels.gift")}
			</Button>
			<Button hideOnDesktop>
				{#snippet icon()}
					<Gift />
				{/snippet}
			</Button>

			<DropDown>
				{#snippet dropdown()}
					<Button big href="/cosmetics">
						{#snippet icon()}
							<PaintBrush />
						{/snippet}
						Your Cosmetics
					</Button>
					<Button big style="color: var(--danger)">
						{#snippet icon()}
							<Warning />
						{/snippet}
						Cancel Subscription
					</Button>
				{/snippet}
				<Button secondary>
					{#snippet icon()}
						<DotsThreeVertical />
					{/snippet}
				</Button>
			</DropDown>
		</div>
	{/snippet}
	<SubInfo />
</StoreSection>

<style lang="scss">
	.buttons {
		display: flex;
		gap: 0.5rem;
	}
</style>
