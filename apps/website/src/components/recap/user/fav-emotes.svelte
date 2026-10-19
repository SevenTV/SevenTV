<script lang="ts">
	type RecapMode = "global" | "user";

	type potatEmoteFormat = {
		id: string;
		name: string;
		alias: string;
		provider: string;
		use_count: number;
		urls: { url: string; mime: string; size: number; scale: number }[];
	};

	interface Props {
		favUserEmotes: { sum_used: number; top_used: potatEmoteFormat[] };
		favChannelEmotes: { sum_used: number; top_used: potatEmoteFormat[] };
	}

	let { favChannelEmotes, favUserEmotes }: Props = $props();
	let PickedRecapMode = $state<RecapMode>("global");
	let isChannelDataAvailable = favChannelEmotes && favChannelEmotes.top_used?.length > 0;

	const leaderboardUser = favUserEmotes?.top_used?.map((item, index) => ({
		rank: index + 1,
		name: item.name,
		alias: item.alias,
		id: item.id,
		image: `https://cdn.7tv.app/emote/${item.id}/4x.webp`,
		score: item.use_count,
	}));

	const leaderboardChannel = favChannelEmotes?.top_used?.map((item, index) => ({
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
		<h1 class="recap title1">YOUR FAVOURITE</h1>
		<h1 class="recap title2">EMOTES*</h1>
		<p class="recap channel-label">*Tracked across over 7K channels</p>
		<div class="section-buttons">
			<button
				class="recap global {PickedRecapMode === 'global' ? 'active' : ''}"
				style={isChannelDataAvailable ? "" : "margin-left: 2rem;"}
				onclick={() => (PickedRecapMode = "global")}
			>
				Global
			</button>
			{#if isChannelDataAvailable}
				<button
					class="recap user {PickedRecapMode === 'user' ? 'active' : ''}"
					onclick={() => (PickedRecapMode = "user")}>Your Channel</button
				>
			{/if}
		</div>
		<div class="leaderboard">
			{#if PickedRecapMode === "global"}
				{#each leaderboardUser.slice(0, 10) as item}
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
									alt="{item.alias !== '' ? item.alias : item.name}'s avatar"
									style="height: 2rem; max-width :3rem; border-radius: 4px;"
								/></a
							>
						</div>
						<span class="name"
							><a
								style="color: white;"
								href={`https://7tv.app/emotes/${item.id}`}
								target="_blank"
								rel="noopener noreferrer">{item.alias !== '' ? item.alias : item.name}</a
							></span
						>
						<span class="score">{item.score.toLocaleString()}</span>
					</div>
				{/each}
			{:else if PickedRecapMode === "user"}
				{#each leaderboardChannel.slice(0, 10) as item}
					<div class="row">
						<span class="rank">{item.rank}.</span>
						<div>
							<a
								style="color: white; width: fit-content;"
								href={`https://7tv.app/emotes/${item.id}`}
								target="_blank"
								rel="noopener noreferrer"
								><img
									src={item.image}
									alt="{item.alias !== '' ? item.alias : item.name}'s avatar"
									style="height: 2rem; max-width :3rem; border-radius: 4px;"
								/></a
							>
						</div>
						<span class="name"
							><a
								style="color: white;"
								href={`https://7tv.app/emotes/${item.id}`}
								target="_blank"
								rel="noopener noreferrer">{item.alias !== '' ? item.alias : item.name}</a
							></span
						>
						<span class="score">{item.score.toLocaleString()}</span>
					</div>
				{/each}
			{/if}
		</div>
	</header>
</section>

<style lang="scss">
	.section-buttons {
		margin-top: 5rem;
		display: flex;
		margin-bottom: 3rem;
		gap: 8rem;
	}
	.recap {
		font-size: 7rem;
		margin: 0;
		font-family: "BBH Bartleby", sans-serif;
		font-weight: 200;

		&.title1 {
			color: #ffffff;
			font-size: 2.5rem;

			letter-spacing: 0.3rem;
		}
		&.title2 {
			color: #ff3f7f;
			font-size: 7.5rem;

			letter-spacing: 0.3rem;
		}
		&.channel-label {
			font-size: 1rem;
			font-family: "Syne Bold", sans-serif;
			color: #ffffff;
		}
		&.global {
			font-size: 2.5rem;
			margin-right: 1rem;
		}

		&.active {
			color: #836fff;
		}

		&.user {
			font-size: 2.5rem;
		}
	}
	.hero {
		position: relative;
		min-height: 60rem;
		color: #fff;

		background-color: #1d1d1d;
		background-image: url("../../../../assets/user-recap-shapes2.png");
		background-repeat: no-repeat;
		background-position: center center;
		background-size: cover;

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
	}

	.leaderboard {
		width: 40rem;
		background: #8576ff;
		padding: 2rem;
		margin: 1rem;
		display: flex;
		flex-direction: column;
		gap: 14px;
		border: 1rem solid #ffffff;
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
		.section-buttons {
			margin-top: 2rem;
			display: flex;
			margin-bottom: 2rem;
			gap: 0rem;
		}
		.recap {
			&.title1 {
				font-size: 2rem;
			}
			&.title2 {
				font-size: 5rem;
			}
			&.global {
				font-size: 1rem;
			}
			&.user {
				font-size: 1rem;
			}
		}
		.hero {
			min-height: 40rem;
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
		.leaderboard {
			width: 100%;
			max-width: 32rem;
			padding: 1.2rem;
		}
		.row {
			grid-template-columns: 24px 12rem 1fr auto;
			gap: 16px;
			.name {
				font-size: 0.7rem;
			}
			.rank {
				font-size: 0.7rem;
			}
		}
	}

	@media (max-width: 900px) {
		.recap {
			&.title1 {
				font-size: 1.3rem;
			}
			&.title2 {
				font-size: 2.5rem;
			}
		}
		.hero {
			min-height: 25rem;
		}
		.top {
			margin-top: 2rem;
			margin-left: 0.5rem;
		}
		.leaderboard {
			width: 100%;
			max-width: 100%;
			padding: 0.7rem;
			border-width: 0.5rem;
		}
		.row {
			gap: 10px;
			grid-template-columns: 40px 59px 1fr auto;
			.name,
			.score,
			.rank {
				font-size: 0.7rem;
			}
		}
	}

	@media (max-width: 600px) {
		.recap {
			&.title1 {
				font-size: 1rem;
			}
			&.title2 {
				font-size: 1.5rem;
			}
		}
		.hero {
			min-height: 15rem;
		}
		.top {
			margin-top: 1rem;
			margin-left: 0;
		}
		.leaderboard {
			padding: 0.3rem;
			border-width: 0.3rem;
			gap: 8px;
		}
		.row {
			gap: 6px;
			grid-template-columns: 40px 59px 1fr auto;
		}
	}
</style>
