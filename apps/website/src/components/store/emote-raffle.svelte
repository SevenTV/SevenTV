<script lang="ts">
	import Button from "../input/button.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretLeft, CaretRight, Crown, Ticket } from "phosphor-svelte";
	import { t } from "svelte-i18n";
</script>

<StoreSection title={$t("pages.store.subscription.raffle")}>
	{#snippet header()}
		<span class="header">Dec 2023</span>
	{/snippet}
	<div class="carousel">
		<Button>
			{#snippet icon()}
				<CaretLeft />
			{/snippet}
		</Button>
		<!-- <EmotePreview bg="light" /> -->
		<Button>
			{#snippet icon()}
				<CaretRight />
			{/snippet}
		</Button>
	</div>
	<div class="footer">
		<div class="winner">
			<div class="profile-picture"></div>
			<span>Pokimane</span>
			<Crown color="var(--store)" />
		</div>
		<Button secondary>
			{#snippet icon()}
				<Ticket />
			{/snippet}
			{$t("labels.enter")}
		</Button>
	</div>
</StoreSection>

<style lang="scss">
	.header {
		color: var(--text-light);
		font-size: 0.875rem;
		font-weight: 500;
	}

	.carousel {
		flex-grow: 1;

		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.winner {
		display: flex;
		align-items: center;
		gap: 0.75rem;

		font-size: 0.875rem;
		font-weight: 500;

		.profile-picture {
			width: 1.5rem;
			height: 1.5rem;
			background-color: var(--secondary);
			border-radius: 50%;
		}
	}
</style>
