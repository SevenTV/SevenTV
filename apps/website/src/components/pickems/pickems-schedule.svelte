<script lang="ts">
	import { Ticket } from "phosphor-svelte";
	import Button from "../input/button.svelte";

	let { hasPass }: { hasPass: Boolean } = $props();

	let schedule = [
		{
			day: "Day 1",
			date: "Feb. 29th",
			stage: "Round of 16",
			bestOf: "1 Match",
			startTime: "10:00 AM E.T / 3:00 PM GMT",
		},
		{
			day: "Day 2",
			date: "Mar. 1st",
			stage: "Quarterfinals",
			bestOf: "1 Match",
			startTime: "10:00 AM E.T / 3:00 PM GMT",
		},
		{
			day: "Day 3",
			date: "Mar. 2nd",
			stage: "Semifinals + Finals + Showmatch",
			bestOf: "3 Matches",
			startTime: "10:00 AM E.T / 3:00 PM GMT",
		},
	];
</script>

<div class="container">
	<div class="header">
		<h1>
			Tournament<br />Schedule
		</h1>
	</div>

	<div class="schedule-grid">
		{#each schedule as match}
			<div class="match-card">
				<div class="match-header">
					<span>{match.day}</span>
					<span>{match.date}</span>
				</div>
				<div class="match-details">
					<div class="match-row">
						<span>Stage</span>
						<span>{match.stage}</span>
					</div>
					<div class="match-row">
						<span>Best of</span>
						<span>{match.bestOf}</span>
					</div>
					<div class="match-row">
						<span>Start Time</span>
						<span>{match.startTime}</span>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>

<style scss>
	h1 {
		font-size: 1.5rem;
		font-weight: normal;
		font-family: Inter;
		font-size: 48px;
		font-style: normal;
		background: linear-gradient(0deg, #fff -24.36%, rgba(255, 255, 255, 0.61) 116.67%);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
	}
	.container {
		border-radius: 12px;
		border: 1px solid rgba(255, 255, 255, 0.15);
		background: black;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 2rem;
		color: white;
		width: 100%;

		.header {
			display: flex;
			justify-content: space-between;
			align-items: center;
			width: 100%;
			margin-bottom: 1.5rem;

			h1 {
				font-size: 2rem;
				font-weight: bold;
			}
		}

		.schedule-grid {
			display: grid;
			grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
			gap: 1.5rem;
			width: 100%;

			.match-card {
				background: rgba(235, 219, 255, 0.11);
				border-radius: 8px;
				border: 1px solid #413c47;

				padding: 1rem;

				.match-header {
					display: flex;
					justify-content: space-between;
					background: #ebdbff;
					border-radius: 8px;
					border: 1px solid #413c47;
					padding: 0.5rem;
					font-weight: bold;
					color: #2d2d2d;
				}

				.match-details {
					margin-top: 1rem;

					.match-row {
						display: flex;
						justify-content: space-between;
						border-bottom: 1px solid #444;
						padding: 0.5rem 0;
						color: #ccc;
					}
				}
			}
		}
	}
</style>
