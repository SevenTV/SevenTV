<script lang="ts">
	interface Props {
		userDataBadgesCount: string;
		userDataPaintsCount: string;
	}

	let { userDataPaintsCount, userDataBadgesCount }: Props = $props();
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div>
				<h1 class="recap title left">{userDataBadgesCount}</h1>
				<h1 class="recap title emotes left">BADGES</h1>
				<h1 class="recap title sent left">CHANGES</h1>
			</div>
			<div>
				<p class="recap subtitle">You totally crushed it</p>
				<p class="recap subtitle">as the most stylish</p>
				<p class="recap subtitle">user in the chat!</p>
			</div>
			<div>
				<h1 class="recap title right">{userDataPaintsCount}</h1>
				<h1 class="recap title emotes right">PAINTS</h1>
				<h1 class="recap title sent right">CHANGES</h1>
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
		&.left {
			text-align: left;
			color: #836fff;
		}
		&.right {
			text-align: right;
			color: #ff3f7f;
		}
		&.title {
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

		&.subtitle {
			font-size: 1.5rem;
			font-weight: 1000;
			font-family: "Syne Bold", sans-serif;
			color: #1d1d1d;
			text-align: center;
		}
	}
	.side-by-side {
		display: flex;
		align-items: center;
		gap: 3rem;
	}
	.hero {
		position: relative;
		min-height: 48rem;
		color: #fff;
		background-color: #ffffff;
		background-image: url("../../../../assets/user-recap-shapes5.png");
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

			&.left {
				text-align: center;
			}
			&.right {
				text-align: center;
			}

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
			background-color: #ffffff;
			background-image: unset;
			background-repeat: no-repeat;
			background-position: center center;
			background-size: cover;
		}
		.top {
			margin-top: 5rem;
			margin-left: 1rem;
		}
	}

	@media (max-width: 900px) {
		.recap {
			font-size: 2.5rem;

			&.left {
				text-align: center;
			}
			&.right {
				text-align: center;
			}

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
