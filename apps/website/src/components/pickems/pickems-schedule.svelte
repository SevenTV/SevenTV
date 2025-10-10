<script lang="ts">
	import { t } from "svelte-i18n";
	let schedule = [
		{
			day: "Day 1",
			date: "Feb. 28th",
			stage: "Round of 16",
			bestOf: "1 Map",
			startTime: "10:00 AM EST / 3:00 PM GMT",
		},
		{
			day: "Day 2",
			date: "Mar. 1st",
			stage: "Quarterfinals",
			bestOf: "1 Map",
			startTime: "12:00 PM EST / 5:00 PM GMT",
		},
		{
			day: "Day 3",
			date: "Mar. 2nd",
			stage: "Semifinals + Finals + Showmatch",
			bestOf: "3 Maps",
			startTime: "10:00 AM EST / 3:00 PM GMT",
		},
	];
</script>

<div class="container">
	<div class="header">
		<h1>{$t("pages.store.events.cs2.schedule.header")}</h1>
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
						<span>{$t("pages.store.events.cs2.schedule.stage")}</span>
						<span>{match.stage}</span>
					</div>
					<div class="match-row">
						<span>{$t("pages.store.events.cs2.schedule.best_of")}</span>
						<span>{match.bestOf}</span>
					</div>
					<div class="match-row">
						<span>{$t("pages.store.events.cs2.schedule.start_time")}</span>
						<span>{match.startTime}</span>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>

<style>
	h1 {
		font-weight: normal;
		font-family: Inter, serif;
		font-size: 48px;
		font-style: normal;
		background: linear-gradient(0deg, #fff -24.36%, rgba(255, 255, 255, 0.61) 116.67%);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		text-align: center;
	}
	.container {
		background-color: transparent;
		padding: 3rem 3rem;
		border-radius: 12px;
		border: 1px solid rgba(255, 255, 255, 0.15);
		display: flex;
		flex-direction: column;
		align-items: center;
		color: white;
		width: 100%;

		.header {
			display: flex;
			justify-content: center;
			align-items: center;
			width: 100%;
			margin-bottom: 1.5rem;

			h1 {
				font-size: 2.5rem;
				font-weight: bold;
			}

			@media screen and (max-width: 1200px) {
				h1 {
					font-size: 2rem;
				}
			}
		}

		.schedule-grid {
			display: flex;
			/*grid-template-columns: repeat(auto-fit, minmax(350px, 3fr));*/
			flex-direction: row;
			gap: 1.5rem;
			justify-content: space-between;
			width: 100%;

			.match-card {
				background: rgba(235, 219, 255, 0.11);
				border-radius: 8px;
				border: 1px solid #413c47;
				padding: 0;
				overflow: hidden;
				width: 100%;

				.match-header {
					padding: 1rem;
					font-size: 1.2rem;
					border: none;
					display: flex;
					justify-content: space-between;
					background: #ebdbff;
					font-weight: bold;
					color: #2d2d2d;
				}

				.match-details {
					margin-top: 0.5rem;

					.match-row {
						padding: 1rem 1.2rem;
						display: flex;
						justify-content: space-between;
						color: #ccc;
						flex-direction: row;
					}

					.match-row span:first-child {
						font-weight: bold;
					}

					@media screen and (max-width: 500px) {
						.match-row {
							flex-direction: column !important;
						}
					}
				}
			}
		}

		@media screen and (max-width: 1600px) {
			.schedule-grid {
				flex-direction: column;
				width: 100%;
			}
		}
	}
</style>
