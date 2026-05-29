<script lang="ts">

    interface Props {
        topChannelsData: Array<{ channel: {
            id: string;
            login: string;
            display: string;
            chat_color: string;
            pfp: string;
        }; count: number; }>;
    }

    let { topChannelsData }: Props = $props();

    const leaderboard = topChannelsData.map((item, index) => ({
        rank: index + 1,
        name: item.channel?.display,
        pfp: item.channel?.pfp,
        chatColor: item.channel?.chat_color,
        twitchLink: `https://twitch.tv/${item.channel?.login}`,
        score: item.count,
    }));
</script>

<section class="hero">
	<header class="top">
		<h1 class="recap title1">TOP CHANNELS</h1>
		<h1 class="recap title2">OF 7TV</h1>
		<div style="margin-top: 3rem;">
			<p class="recap subtitle">7TV emotes are really loved in these channels.</p>
			<p class="recap subtitle">They even have the lead of the most</p>
			<p class="recap subtitle">spammed 7TV emotes in all channels!</p>
		</div>
		<div class="leaderboard">
            {#each leaderboard as item}
                <div class="row">
                    <span class="rank">{item.rank}.</span>
                    <div class="avatar">
                        <img src={item.pfp} alt="{item.name}'s avatar" style="width: 100%; height: 100%; border-radius: 4px;" />
                    </div>
                    <span class="name"><a href={item.twitchLink} target="_blank" rel="noopener noreferrer" style="color: {item.chatColor};">{item.name}</a></span>
                    <span class="score">{item.score.toLocaleString()}</span>
                </div>
            {/each}
		</div>
	</header>
</section>

<style lang="scss">
	.recap {
		font-size: 7rem;
		margin: 0;
		font-family: "BBH Bartleby", sans-serif;
		font-weight: 200;

		&.title1 {
			color: #000000;
			font-size: 2.5rem;

			letter-spacing: 0.3rem;
		}
		&.title2 {
			color: #000000;
			font-size: 7.5rem;

			letter-spacing: 0.3rem;
		}

		&.subtitle {
			font-size: 1.5rem;
			font-weight: 1000;
			text-align: center;
			font-family: "Syne Bold", sans-serif;
			color: #1d1d1d;
		}
	}
	.hero {
		position: relative;
		min-height: 60rem;
		color: #fff;

		background-color: #aed2ff;
		background-image: url("../../../../assets/recap-shapes.png");
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
		width: min(60rem, 95vw);
		max-width: 95vw;
		background: #a6ff96;
		padding: 2rem;
		height: 40rem;
		display: flex;
		margin-top: 2rem;
		overflow-y: auto;
		margin-bottom: 2rem;
		flex-direction: column;
		gap: 14px;
		border: 1rem solid #000000;
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
			color: #000000;
		}

		.avatar {
			width: 3.3rem;
			height: 3.3rem;
			background: #000000;
			border-radius: 4px;
		}

		.name {
			font-size: 1.1rem;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
			font-family: "Syne ExtraBold", sans-serif;
			font-weight: 900;
			letter-spacing: 0.1rem;
			color: #000000;
		}

		.score {
			font-size: 1rem;
			letter-spacing: 0.3px;
			color: #000000;
		}
	}

	@media (max-width: 1500px) {
		.hero {
			min-height: 40rem;
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
		.leaderboard {
			width: min(40rem, 98vw);
			height: 30rem;
			padding: 1rem;
			border-width: 0.7rem;
		}
		.recap.title2 {
			font-size: 4rem;
		}
		.row{
			.name {
				font-size: 0.7rem;
			}
			.rank {
				font-size: 0.7rem;
			}
		}
	}

	@media (max-width: 900px) {
		.hero {
			min-height: 30rem;
		}
		.top {
			margin-top: 2rem;
			margin-left: 0.5rem;
		}
		.leaderboard {
			width: 98vw;
			height: 22rem;
			padding: 0.7rem;
			border-width: 0.5rem;
		}
		.recap.title1 {
			font-size: 1.5rem;
		}
		.recap.title2 {
			font-size: 2.5rem;
		}
		.recap.subtitle {
			font-size: 1rem;
		}
		.row {
			grid-template-columns: 22px 28px 1fr auto;
			gap: 12px;
			.avatar {
				width: 2.2rem;
				height: 2.2rem;
			}
			.name {
				font-size: 0.9rem;
			}
			.score {
				font-size: 0.9rem;
			}
		}
	}

	@media (max-width: 600px) {
		.hero {
			min-height: 20rem;
		}
		.top {
			margin-top: 1rem;
			margin-left: 0;
		}
		.leaderboard {
			width: 100vw;
			height: 15rem;
			padding: 0.3rem;
			border-width: 0.3rem;
		}
		.recap.title1 {
			font-size: 1rem;
		}
		.recap.title2 {
			font-size: 1.3rem;
		}
		.recap.subtitle {
			font-size: 0.8rem;
		}
		.row {
			grid-template-columns: 30px 20px 1fr auto;
			gap: 6px;
			.avatar {
				width: 1.3rem;
				height: 1.3rem;
			}
			.name {
				font-size: 0.7rem;
			}
			.score {
				font-size: 0.7rem;
			}
		}
	}
</style>
