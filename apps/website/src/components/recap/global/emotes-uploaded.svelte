<script lang="ts">
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div>
				<h1 class="recap title">523,078</h1>
				<h1 class="recap title emotes">EMOTES</h1>
				<h1 class="recap title uploaded">uploaded</h1>
			</div>
			<div>
				<p class="recap subtitle">7tv just keeps getting</p>
				<p class="recap subtitle">bigger, and it doesn't look</p>
				<p class="recap subtitle">like it's slowing down!</p>
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
			color: #836fff;
			&.emotes {
				color: white;
				font-weight: 100;
				font-size: 5rem;
				-webkit-text-stroke: 3px #1D1D1D;
			}
			&.uploaded {
				font-size: 5rem;
				color: #1D1D1D;
			}
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
			&.title.uploaded {
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
			&.title.uploaded {
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
