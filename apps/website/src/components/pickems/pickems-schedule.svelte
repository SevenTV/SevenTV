<script lang="ts">
	interface Props {
		onShowTooltip: (stage: string) => void;
		onHideTooltip: () => void;
	}

	let { onShowTooltip, onHideTooltip }: Props = $props();
	let schedule = [
		{
			day: "Day 1",
			date: "October 23",
			stages: [
				"EU Upper Bracket – Round 1 & Round 2",
				"EU Lower Bracket – Round 1 (2 elimination matches)",
				"NA Upper Bracket – Round 1 & Round 2",
			],
			startTime: "11:30 AM CEST / 5:30 AM ET",
		},
		{
			day: "Day 2",
			date: "October 24",
			stages: ["EU Lower Bracket – Round 2 & Round 3", "NA Lower Bracket – Round 1 & Round 2"],
			startTime: "11:30 AM CEST / 5:30 AM ET",
		},
		{
			day: "Day 3",
			date: "October 25",
			stages: [
				"EU Upper Bracket Semifinal + Lower Bracket Round 4",
				"NA Upper Bracket Semifinal + Lower Bracket Round 4",
			],
			startTime: "11:30 AM CEST / 5:30 AM ET",
		},
		{
			day: "Day 4",
			date: "October 26",
			stages: ["NA & EU Finals", "Showmatch", "Grand Final"],
			startTime: "11:30 AM CET / 6:30 AM ET",
		},
	];
</script>

<div class="container">
	<div class="header">
		<h1>Tournament Schedule</h1>
	</div>

	<div class="schedule-grid">
		{#each schedule as match}
			<div class="match-card">
				<div class="match-header">
					<span>{match.day}</span>
					<span>{match.date}</span>
				</div>
				<div class="match-details">
					<div class="match-row stages">
						{#each match.stages as stage, i}
							<div
								class="stage-item"
								onmouseenter={() => onShowTooltip(stage)}
								onmouseleave={onHideTooltip}
								aria-label={stage}
								role="button"
								tabindex="0"
							>
								<span
									style="text-decoration: underline; color: #a78bfa; cursor: pointer; transition: color 0.2s;"
								>
									Stage {i + 1}
								</span>
							</div>
						{/each}
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
		.stages {
			margin-bottom: 1rem;
			flex-direction: column;
			align-items: flex-start;
		}
		.stages span:not(:first-child) {
			display: block;
			margin-left: 1.5rem;
		}

		.stage-item {
			position: relative;
			display: inline-block;
			margin-right: 1rem;
			cursor: pointer;
		}
		.stage-item:last-child {
			margin-right: 0;
		}
		.stage-item:hover span {
			text-decoration: underline;
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
