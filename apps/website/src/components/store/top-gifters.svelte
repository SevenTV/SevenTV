<script lang="ts">
	import { numberFormat } from "$/lib/utils";
	import Button from "../input/button.svelte";
	import StoreSection from "./store-section.svelte";
	import { CaretLeft, CaretRight, Trophy, Gift } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	const topGifters = [
		{
			name: "forsen",
			amount: 2100000500,
		},
		{
			name: "CupOfEggy",
			amount: 1640,
		},
		{
			name: "Pokimane",
			amount: 121,
		},
		{
			name: "ayyybubu",
			amount: 89,
		},
		{
			name: "xQc",
			amount: 56,
		},
	];
</script>

<StoreSection title={$t("pages.store.subscription.top_gifters")}>
	{#snippet header()}
		<div class="header">
			<Button>
				{#snippet icon()}
					<CaretLeft />
				{/snippet}
			</Button>
			<Button>
				{#snippet icon()}
					<CaretRight />
				{/snippet}
			</Button>
		</div>
	{/snippet}
	<div class="grid">
		{#each topGifters as gifter, i}
			<span class="rank">{i + 1}.</span>
			<div class="profile-picture"></div>
			<span class="name">{gifter.name}</span>
			<span class="amount">
				<span class:gold={i === 0} class:silver={i === 1} class:bronze={i === 2}>
					{#if i < 3}
						<Trophy />
					{/if}
				</span>
				{numberFormat().format(gifter.amount)}
			</span>
		{/each}
	</div>
	<Button secondary style="align-self: flex-end">
		{#snippet icon()}
			<Gift />
		{/snippet}
		{$t("labels.gift")}
	</Button>
</StoreSection>

<style lang="scss">
	.header {
		display: flex;
		gap: 0.5rem;
	}

	.grid {
		display: grid;
		grid-template-columns: auto auto 1fr auto;
		align-items: center;
		column-gap: 0.75rem;
		row-gap: 1rem;

		font-size: 0.875rem;
		font-weight: 500;

		& > .rank {
			color: var(--text-light);
		}

		& > .profile-picture {
			width: 1.5rem;
			height: 1.5rem;
			border-radius: 50%;
			background-color: var(--secondary);
		}

		& > .amount {
			display: flex;
			gap: 0.75rem;
			align-items: center;
			justify-content: flex-end;
		}
	}

	.gold {
		color: #eab631;
	}

	.silver {
		color: #c8c8c8;
	}

	.bronze {
		color: #c27924;
	}
</style>
