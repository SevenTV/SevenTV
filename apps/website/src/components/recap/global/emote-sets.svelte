<script lang="ts">
	const leaderboard = [
		{ rank: 1, name: "COPIUM", score: 145109, id: "01F6ME7ADR0000WDA7ERT9H30R" },
		{ rank: 2, name: "catJAM", score: 47490, id: "01F6MQ33FG000FFJ97ZB8MWV52" },
		{ rank: 3, name: "NOOOO", score: 46269, id: "01F6MKTFTG0009C9ZSNZTFV2ZF" },
		{ rank: 4, name: "GIGACHAD", score: 41364, id: "01F6MZGCNG000255K4X1K7NTHR" },
		{ rank: 5, name: "o7", score: 38736, id: "01FS5ZCFG0000500DPPCXJWCP8" },
		{ rank: 6, name: "Cinema", score: 38243, id: "01GBFDVP18000CRDCG0DV7KEMY" },
		{ rank: 7, name: "xdd", score: 36595, id: "01GESGWZGR0008M43N8RKPP1GG" },
		{ rank: 8, name: "catKISS", score: 33430, id: "01F5VW2TKR0003RCV2Z6JBHCST" },
		{ rank: 9, name: "classic", score: 33418, id: "01GBPSCGR00007Q17796BDN5AJ" },
		{ rank: 10, name: "SNIFFA", score: 32858, id: "01F7M225F8000AWSXNQ65M4PKG" },
	];
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div class="year-div">
				<h1 class="recap count">16,962,827</h1>
				<h1 class="recap emotes">EMOTES</h1>
				<h1 class="recap added">ADDED TO SETS</h1>
				<p class="recap channel-label">*Tracked across over 7K channels</p>
			</div>
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
									src={`https://cdn.7tv.app/emote/${item.id}/4x.webp`}
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
		text-align: left !important;

		&.count {
			color: #a6ff96;
		}
		&.emotes {
			color: #ffffff;
			font-size: 5rem;
		}
		&.added {
			color: #1d1d1d;
			font-size: 5rem;
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
		margin: 1rem;
		display: flex;
		flex-direction: column;
		gap: 14px;
		border: 1rem solid #a6ff96;
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

	@media (max-width: 1500px) {
		.hero {
			min-height: 40rem;
		}
		.leaderboard {
			width: 100%;
			max-width: 32rem;
			padding: 1.2rem;
			margin: unset;
		}
		.top {
			margin-top: 5rem;
			margin-left: 0;
		}
		.side-by-side {
			flex-direction: column;
			align-items: stretch;
			gap: 2rem;
		}
		.year-div {
			margin-left: 0;
			align-items: flex-start;
		}
		.recap {
			font-size: 4rem;
			&.emotes,
			&.added {
				font-size: 3rem;
			}
			&.added {
				-webkit-text-stroke: 0.5px #fff;
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
			margin: unset;
		}
		.row {
			gap: 10px;
			grid-template-columns: 40px 100px 1fr auto;
			.name,
			.score,
			.rank {
				font-size: 0.7rem;
			}
		}
		.recap {
			font-size: 2.2rem;
			&.emotes,
			&.added {
				font-size: 1.7rem;
			}
			&.added {
				-webkit-text-stroke: 0.5px #fff;
			}
		}
		.top {
			margin-top: 2rem;
		}
	}

	@media (max-width: 600px) {
		.hero {
			min-height: unset;
			padding: 0.5rem;
		}
		.leaderboard {
			padding: 0.3rem;
			border-width: 0.3rem;
			gap: 8px;
			margin: unset;
		}
		.row {
			gap: 6px;
			grid-template-columns: 40px 100px 1fr auto;
		}
		.recap {
			font-size: 1.2rem;
			&.emotes,
			&.added {
				font-size: 1rem;
			}
			&.added {
				-webkit-text-stroke: 0.5px #fff;
			}
			&.channel-label {
				font-size: 0.8rem;
			}
		}
		.top {
			margin-top: 1rem;
		}
	}
</style>
