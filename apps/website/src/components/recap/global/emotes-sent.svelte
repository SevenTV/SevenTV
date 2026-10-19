<script lang="ts">
	interface Props {
		emotesSentData: Array<{
			alias: string;
			id: string;
			name: string;
			provider: string;
			use_count: number;
		}>;
		emoteSentCount: number;
	}

	let { emotesSentData, emoteSentCount }: Props = $props();

	const leaderboard = emotesSentData.map((item, index) => ({
		rank: index + 1,
		name: item.name,
		alias: item.alias,
		id: item.id,
		image: `https://cdn.7tv.app/emote/${item.id}/4x.webp`,
		score: item.use_count,
	}));
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div class="leaderboard">
				{#each leaderboard as item}
					<div class="row">
						<span class="rank">{item.rank}.</span>
						<div>
							<a
								style="color: white;"
								href={`https://7tv.app/emotes/${item.id}`}
								target="_blank"
								rel="noopener noreferrer"
								><img
									src={item.image}
									alt="{item.name}'s avatar"
									style="height: 2rem; max-width :3rem; border-radius: 4px;"
								/></a
							>
						</div>
						<span class="name"
							><a
								style="color: white;"
								href={`https://7tv.app/emotes/${item.id}`}
								target="_blank"
								rel="noopener noreferrer">{item.name}</a
							></span
						>
						<span class="score">{item.score.toLocaleString()}</span>
					</div>
				{/each}
			</div>
			<div class="year-div">
				<h1 class="recap count">{emoteSentCount.toLocaleString()}</h1>
				<h1 class="recap emotes">EMOTES</h1>
				<h1 class="recap sent">SENT*</h1>
				<p class="recap channel-label">*Tracked across over 7K channels</p>
			</div>
		</div>
	</header>
</section>

<style lang="scss">
	.year-div {
		display: flex;
		flex-direction: column;
		margin-left: 1.3rem;
	}
	.recap {
		font-size: 6rem;
		margin: 0;
		font-family: "BBH Bartleby", sans-serif;
		font-weight: 200;
		text-align: right !important;

		&.count {
			color: #836fff;
		}
		&.emotes {
			color: #ffffff;
		}
		&.sent {
			color: #1d1d1d;
			-webkit-text-stroke: 4px #fff;
		}
		&.channel-label {
			font-size: 1rem;
			font-family: "Syne Bold", sans-serif;
			color: #ffffff;
		}
	}
	.side-by-side {
		display: flex;
		align-items: center;
	}
	.hero {
		position: relative;
		min-height: 60rem;
		color: #fff;
		background-color: #1d1d1d;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.top {
		margin-top: 10rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		margin-left: 3rem;

		.header-content {
			font-weight: 900;
			font-size: 2rem;
			color: #a970ff;
		}
	}

	.leaderboard {
		width: 40rem;
		background: #8576ff;
		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 14px;
		border: 1rem solid #fff;
		font-family: "BBH Bartleby", sans-serif;
	}
	.row {
		display: grid;
		grid-template-columns: 30px 34px 1fr auto;
		align-items: center;
		gap: 30px;
		color: #ffffff;
		font-weight: 100;

		.rank {
			font-size: 1rem;
			opacity: 0.9;
		}
		.name {
			font-size: 1.1rem;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
			font-family: "Syne ExtraBold", sans-serif;
			font-weight: 900;
			letter-spacing: 0.1rem;
		}

		.score {
			font-size: 1rem;
			letter-spacing: 0.3px;
		}
	}

	// Responsive styles
	@media (max-width: 1500px) {
		.hero {
			min-height: 40rem;
		}
		.leaderboard {
			width: 100%;
			max-width: 32rem;
			padding: 1.2rem;
			border-width: 0.7rem;
		}
		.year-div {
			margin-left: 0.7rem;
		}
		.recap {
			font-size: 4.5rem;
			&.channel-label {
				font-size: 0.95rem;
			}
			&.sent {
				color: #1d1d1d;
				-webkit-text-stroke: 1px #fff;
			}
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
		.side-by-side {
			flex-direction: column;
			align-items: stretch;
			gap: 2rem;
		}
		.row {
			grid-template-columns: 29px 47px 1fr auto;
			gap: 12px;
			.name,
			.score,
			.rank {
				font-size: 0.7rem;
			}
		}
	}

	@media (max-width: 900px) {
		.hero {
			min-height: 20rem;
			padding: 1rem;
		}
		.leaderboard {
			width: 100%;
			max-width: 100%;
			padding: 0.7rem;
			border-width: 0.5rem;
		}
		.year-div {
			margin-left: 0;
			align-items: flex-end;
		}
		.recap {
			font-size: 2.5rem;
			&.channel-label {
				font-size: 0.85rem;
			}
			&.sent {
				color: #1d1d1d;
				-webkit-text-stroke: 1px #fff;
			}
		}
		.top {
			margin-top: 2rem;
			margin-left: 0;
		}
		.side-by-side {
			flex-direction: column;
			align-items: stretch;
			gap: 1.2rem;
		}
		.row {
			grid-template-columns: 29px 47px 1fr auto;
			gap: 12px;
			.name,
			.score,
			.rank {
				font-size: 0.7rem;
			}
		}
	}

	@media (max-width: 600px) {
		.hero {
			min-height: 10rem;
			padding: 0.5rem;
		}
		.leaderboard {
			padding: 0.4rem;
			border-width: 0.3rem;
		}
		.recap {
			font-size: 1.3rem;
			&.channel-label {
				font-size: 0.7rem;
			}
			&.sent {
				color: #1d1d1d;
				-webkit-text-stroke: 1px #fff;
			}
		}
		.row {
			grid-template-columns: 50px 51px 1fr auto;
			gap: 6px;
			.name,
			.score,
			.rank {
				font-size: 0.7rem;
			}
		}
	}
</style>
