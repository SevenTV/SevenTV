<script lang="ts">
	interface Props {
		emoteSentCount: number;
	}

	let { emoteSentCount }: Props = $props();
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div>
				<p class="recap subtitle">Woah, slow down there cowboy.</p>
				<p class="recap subtitle">Our servers can’t</p>
				<p class="recap subtitle">keep up.</p>
			</div>
			<div>
				<h1 class="recap title">{emoteSentCount.toLocaleString()}</h1>
				<h1 class="recap title emotes">EMOTES</h1>
				<h1 class="recap title sent">SENT*</h1>
				<p class="recap channel-label">*Tracked across over 7K channels</p>
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

		&.title {
			color: #FF3F7F;
			&.emotes {
				color: white;
				font-weight: 100;
				font-size: 5rem;
				-webkit-text-stroke: 3px #1d1d1d;
			}
			&.sent {
				font-size: 5rem;
				color: #1d1d1d;
			}
		}
		&.channel-label {
			font-size: 1rem;
			font-family: "Syne Bold", sans-serif;
			color: #000000;
		}

		&.subtitle {
			font-size: 2.5rem;
			font-weight: 1000;
			text-align: center;
			font-family: "Syne Bold", sans-serif;
			color: #1d1d1d;
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
		background-color: #ffffff;
		background-image: url("../../../../assets/user-recap-shapes1.png");
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
			margin-bottom: 5rem;
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
			margin-bottom: 5rem;
		}
	}
</style>
