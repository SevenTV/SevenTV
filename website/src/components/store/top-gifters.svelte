<script lang="ts">
	import Button from "../button.svelte";
import StoreSection from "./store-section.svelte";
	import { CaretLeft, CaretRight, Trophy, Gift } from "phosphor-svelte";

	const topGifters = [
		{
			name: "forsen",
			amount: 215,
		},
		{
			name: "CupOfEggy",
			amount: 164,
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

<StoreSection title="Top Gifters">
	<div class="header" slot="header">
		<Button>
			<CaretLeft slot="icon" />
		</Button>
		<Button>
			<CaretRight slot="icon" />
		</Button>
	</div>
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
				{gifter.amount}
			</span>
		{/each}
	</div>
	<Button primary style="align-self: flex-end">
		<Gift slot="icon" />
		Gift
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
			color: var(--text-lighter);
		}

		& > .profile-picture {
			width: 1.5rem;
			height: 1.5rem;
			border-radius: 50%;
			background-color: var(--primary);
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
