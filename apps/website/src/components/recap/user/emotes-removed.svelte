<script lang="ts">
	interface Props {
		userEmoteRemoves: string;
	}

	let { userEmoteRemoves }: Props = $props();
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">	
			<div>
				<p class="recap subtitle">But didn’t vibe with the </p>
				<p class="recap subtitle">others.</p>
				<h1 class="recap title">{userEmoteRemoves.toLocaleString()}</h1>
				<h1 class="recap title emotes">EMOTES</h1>
				<h1 class="recap title sent">REMOVED*</h1>
				<p class="recap channel-label">*Tracked across all emote-sets</p>
			</div>
		</div>
	</header>
</section>

<style lang="scss">
	.recap {
		font-size: 7rem;
		margin: 0;
		font-family: "BBH Bartleby", sans-serif;
		font-weight: 200;

		text-align: center;

		&.title {
			color: #FF3F7F;
			&.emotes {
				color: #FFFFFF;
				font-weight: 100;
				font-size: 5rem;
			}
			&.sent {
				font-size: 5rem;
				color: #1d1d1d;
				-webkit-text-stroke: 3px #ffffff;
			}
		}
		&.channel-label {
			font-size: 1rem;
			font-family: "Syne Bold", sans-serif;
			color: #ffffff;
		}

		&.subtitle {
			font-size: 2.5rem;
			font-weight: 1000;
			font-family: "Syne Bold", sans-serif;
			color: #ffffff;
		}
	}
	.side-by-side {
		display: flex;
		align-items: center;
		gap: 10rem;
	}
	.hero {
		position: relative;
		min-height: 40rem;
		color: #fff;
		background-color: #1D1D1D;
		background-image: url("../../../../assets/user-recap-shapes4.png");
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
		margin-bottom: 5rem;
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

	// Responsive styles
	@media (max-width: 1500px) {
		.recap {
			font-size: 4.5rem;

			&.title.emotes,
			&.title.sent {
				font-size: 3rem;
			}
			&.subtitle {
				font-size: 1.5rem;
			}
		}
		.side-by-side {
			gap: 4rem;
		}
		.hero {
			min-height: 25rem;
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
	}

	@media (max-width: 900px) {
		.recap {
			font-size: 2.5rem;

			&.title.emotes,
			&.title.sent {
				font-size: 1.5rem;
			}
			&.subtitle {
				font-size: 1rem;
			}
		}
		.side-by-side {
			flex-direction: column;
			gap: 2rem;
		}
		.hero {
			min-height: 15rem;
		}
		.top {
			margin-top: 2rem;
			margin-left: 0;
		}
	}
</style>
