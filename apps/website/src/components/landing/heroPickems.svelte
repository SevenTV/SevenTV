<script lang="ts">
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
	import image1 from "$assets/t2.jpg";
	import cslogo from "$assets/4a.jpg";
	import image2 from "$assets/t3.jpg";
	import { ArrowSquareOut, Minus } from "phosphor-svelte";
</script>

<div>
	<section class="hero">
		<div class="hero-content">
			<div style="margin-top:-20px;">
				<Logo size={8.5 * 16} />
			</div>
			<div class="content">
				<h1 style="padding-bottom: 20px;">
					The Emote Platform <span class="for-all">for All</span>
				</h1>
				<p style="padding-bottom: 10px;">
					Manage hundreds of emotes for your Twitch, Kick or YouTube channels with ease. Enhance
					your chatting experience.
				</p>
			</div>
			<div class="buttons">
				<Button primary style="font-size: 1em;" href="#download">Download</Button>
				<Button style="font-size: 1em;">Learn More</Button>
			</div>
		</div>
		<!--			<img class="hero-image hide-on-mobile" src={landingPagePlaceholder} alt="7TV" />-->
		<div class="hero-content hero-image custom-box">
			<div class="top-info hide-on-mobile">
				<p>16 STREAMERS</p>
				<Minus />
				<p>CS2 HOSTED TOURNAMENT</p>
				<Minus />
				<p>3 DAY EVENT</p>
			</div>
			<div class="content custom-content">
				<div class="tabs"></div>

				<div class="pp-header">
					Place Pick’ems <br /><span class="pp-subheader">Win Badges & Paints.</span>
				</div>
				<div class="thumbs">
					<div class="header-image" style="margin-top: -2.5rem;">
						<img src="../assets/pickems_header_bench.png" alt="Header Bench" />
					</div>
				</div>
				<p class="pp-body">
					<i>xQc, OhnePixel, JasonTheWeen</i> and 13 other streamers go head to head. in a
					<i>3-day</i>
					Single Elimination 7TV Hosted <i>CS2</i> tournament.
				</p>

				<div class="buttons centered">
					<Button primary href="/landing/pickems_pass/about">
						<span style=" font-size: 1rem;">Check It Out</span>
					</Button>
					<div class="tdateouter">
						<div class="theader">Event Date</div>
						<div class="tdate">28.02.25 - 02.03.25</div>
					</div>
				</div>
			</div>
		</div>
	</section>
</div>

<style lang="scss">
	.header-image {
		object-fit: contain;
		justify-content: center;
		z-index: 1;
		pointer-events: none;
		bottom: 0;
		img {
			display: inline;
			width: 100%;
			position: relative;
			margin-bottom: -13.5%;
			height: 100%;
		}
	}
	.top-info {
		opacity: 50%;
		display: flex;
		justify-content: center;
		gap: 1rem;

		p {
			font-size: 1rem !important;
		}
	}
	.tdateouter {
		margin-left: auto;
		text-align: center;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		color: #ffa200;
		@media screen and (max-width: 960px) {
			margin-left: 0;
		}
	}

	.theader {
		font-size: 0.72rem;
		font-family: AKONY, serif;
	}

	.s-image {
		width: 132px;
		height: 132px;
		border-radius: 1rem;
	}

	.custom-box {
		border: none;
		padding: 1.2rem;
		border-radius: 1rem;
		background-position: center;
		background-repeat: no-repeat;
		background-size: cover;
		background: rgba(0, 0, 0, 0.5);
	}

	.thumbs {
		margin-bottom: 5.9rem;
		width: 100%;
		justify-content: space-between;
		@media screen and (max-width: 960px) {
			margin-bottom: 1.2rem;
		}
	}

	.custom-button {
		font-size: 1.2em;
		display: inline-block !important;
	}

	.centered {
		justify-content: center;
	}

	.pp-header {
		opacity: 0.9;
		background: linear-gradient(0deg, #fff 24.23%, rgba(255, 255, 255, 0.61) 100%);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		font-size: 1.3rem !important;
		margin-bottom: 2rem;
		font-weight: bold;
		text-align: center;
		font-family: AKONY, serif;
		letter-spacing: 0.1rem;
		line-height: 1.3rem;
	}

	.pp-subheader {
		font-size: 1rem !important;
		color: #eee;
	}

	.pp-body {
		color: white;
		text-align: center;
	}

	.pp-body i {
		color: #ffa200;
	}

	.custom-box h1 {
		margin-bottom: 20px;
	}
	.custom-box p {
		margin-bottom: 20px;
		font-size: 1.125rem;
		line-height: 1.6rem;
	}

	.hero {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 2rem;
		flex-wrap: wrap;

		padding: 5.5rem;
		min-height: min(50rem, 80vh);

		position: relative;
		z-index: 1;

		&.nnys {
			color: var(--text);

			&::before {
				background: url("$assets/nnys-background.webp?url");
				background-size: cover;

				content: "";
				position: absolute;
				top: 0;
				left: 0;
				right: 0;
				bottom: 0;
				border-radius: 1rem;

				z-index: -1;

				mask-image: radial-gradient(
					180% 80% at 0% 100%,
					rgba(white, 1) 0%,
					rgba(white, 0.7) 25%,
					rgba(white, 0.5) 56%,
					rgba(white, 0.2) 79%,
					transparent 100%
				);
				mask-size: 100% 400%;
				animation: fade-in 0.5s linear forwards;
			}

			.hero-content {
				background-color: #00000027;
				backdrop-filter: blur(1rem);
				border-radius: 1rem;
			}
		}

		&::before {
			content: "";
			position: absolute;
			top: 0;
			left: 0;
			right: 0;
			bottom: 0;
			border-radius: 1rem;

			z-index: -1;

			background: radial-gradient(
					180% 80% at 0% 100%,
					#ffffff 0%,
					#5bf8f9 25%,
					#8a00ffe6 56%,
					#25158480 79%,
					transparent 100%
				),
				#1111110a;

			mask-image: linear-gradient(transparent, white);
			mask-image: radial-gradient(
				180% 80% at 0% 100%,
				rgba(white, 1) 0%,
				rgba(white, 0.7) 25%,
				rgba(white, 0.5) 56%,
				rgba(white, 0.2) 79%,
				transparent 100%
			);
			mask-size: 100% 400%;
			animation: fade-in 0.5s linear forwards;
		}

		@keyframes fade-in {
			from {
				mask-position: 0% 0%;
			}
			to {
				mask-position: 0% 100%;
			}
		}
	}

	.hero-content {
		margin-inline: auto;

		display: flex;
		flex-direction: column;
		gap: 1.25rem;
		padding: 2rem;
		background: rgba(0, 0, 0, 0.5);
		border-radius: 1rem;

		.content {
			h1 {
				text-transform: uppercase;

				font-size: 2rem;
				font-weight: 700;
				letter-spacing: 0.15rem;
				text-align: justify;
				text-justify: inter-character;
				word-break: break-word;

				max-width: 27rem;

				font-family: "AKONY", sans-serif;

				.for-all {
					letter-spacing: calc(0.15rem + 2px);
					color: transparent;
					font-size: 2.65rem;
					-webkit-text-stroke-width: 1.5px;
					-webkit-text-stroke-color: var(--text);
				}
			}

			p {
				font-size: 1.125rem;
				font-weight: 400;
				line-height: 1.6rem;

				max-width: 37rem;
			}
		}

		.buttons {
			display: flex;
			flex-wrap: wrap;
			column-gap: 1rem;
			row-gap: 0.5rem;
		}
	}

	.hero-image {
		width: 100%;
		min-width: 15rem;
		max-width: 40rem;

		margin-inline: auto;
	}

	.gifting {
		align-items: center;
		text-align: center;

		// only temporary
		h1 {
			text-align: center !important;
			max-width: unset !important;
		}
	}

	@media screen and (max-width: 960px) {
		.hero {
			padding: 2rem;
		}

		.hero-content .content {
			h1 {
				font-size: 1.8rem;
				text-align: left;

				.for-all {
					font-size: 2rem;
				}
			}

			p {
				font-size: 1rem;
			}
		}
	}
</style>
