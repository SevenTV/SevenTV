<script lang="ts">
	import Banner from "$/components/store/banner.svelte";
	import Benefits from "$/components/store/benefits.svelte";

	let subbed = false;
</script>

<svelte:head>
	<title>Subscription - 7TV</title>
</svelte:head>

<!-- All things called grid here aren't actually css grids -->
<div class="grid">
	<Banner title={subbed ? "Thank You For the Support" : "Unlock Special Perks"} subtitle={subbed ? "Enjoy Your Special Subscriber Perks!" : "Subscribe to ehnance your chatting experience."} />
	{#if !subbed}
		<Benefits />
	{/if}
	<div class="top-grid">
		<div class="subgrid">
			<section class="your-sub">
				<button class="button primary" on:click={() => (subbed = !subbed)}>{subbed ? "Unsub" : "Sub"}</button>
			</section>
			<section class="badge-progress">Badge progress</section>
		</div>
		<section class="monthly-paints">Monthly paints</section>
	</div>
	<div class="three-grid">
		<section class="emote-raffle">Emote raffle</section>
		<section class="personal-emotes">Personal emotes</section>
		<section class="top-gifters">Top gifters</section>
	</div>
	{#if subbed}
		<Benefits />
	{/if}
	<section class="faq">FAQ</section>
</div>

<style lang="scss">
	.grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		flex-wrap: wrap;
	}

	section {
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
		padding: 1rem;
		min-width: 12rem;
	}

	.top-grid {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;

		& > * {
			flex-grow: 1;
		}

		& > .subgrid {
			display: flex;
			flex-direction: column;
			gap: 1rem;
			flex-wrap: wrap;

			& > * {
				flex-grow: 1;
			}
		}
	}

	.three-grid {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;

		& > * {
			flex-grow: 1;
		}
	}
</style>
