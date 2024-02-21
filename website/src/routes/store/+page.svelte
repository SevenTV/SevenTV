<script lang="ts">
	import BadgeProgress from "$/components/store/badge-progress.svelte";
	import Banner from "$/components/store/banner.svelte";
	import Benefits from "$/components/store/benefits.svelte";
	import EmoteRaffle from "$/components/store/emote-raffle.svelte";
	import Faq from "$/components/store/faq.svelte";
	import MonthlyPaints from "$/components/store/monthly-paints.svelte";
	import PersonalEmotes from "$/components/store/personal-emotes.svelte";
	import TopGifters from "$/components/store/top-gifters.svelte";
	import YourSub from "$/components/store/your-sub.svelte";

	let subbed = false;
</script>

<svelte:head>
	<title>Subscription - 7TV</title>
</svelte:head>

<!-- All things called grid here aren't actually css grids -->
<Banner title={subbed ? "Thank You For the Support" : "Unlock Special Perks"} subtitle={subbed ? "Enjoy Your Special Subscriber Perks!" : "Subscribe to ehnance your chatting experience."} />
<div class="grid">
	{#if !subbed}
		<Benefits />
	{/if}
	<div class="top-grid">
		<div class="subgrid">
			<YourSub bind:subbed={subbed} />
			<BadgeProgress />
		</div>
		<MonthlyPaints />
	</div>
	<div class="three-grid">
		<EmoteRaffle />
		<PersonalEmotes />
		<TopGifters />
	</div>
	{#if subbed}
		<Benefits />
	{/if}
	<Faq />
</div>

<style lang="scss">
	.grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		flex-wrap: wrap;

		max-width: 80rem;
		margin: 0 auto;
		margin-top: 1rem;
	}

	.top-grid {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;

		& > .subgrid {
			flex-grow: 1;

			display: flex;
			flex-direction: column;
			gap: 1rem;
			flex-wrap: wrap;
		}
	}

	.three-grid {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
	}
</style>
