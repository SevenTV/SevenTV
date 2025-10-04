<script lang="ts">
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
	import landingPagePlaceholder from "$assets/landing-page-placeholder.webp?url";
	import nnysPlaceholder from "$assets/nnys.webp?url";
	import { isXmasEvent } from "$/lib/xmas";
	import moment from "moment";
	import { user } from "$/lib/auth";
	import { ArrowSquareOut, Ticket } from "phosphor-svelte";
	// import pickemsBanner from "$assets/pickems-banner.png?url";

	let hasPass = $derived(($user?.inventory.products.length ?? 0) > 0);
</script>

<div>
	{#if isXmasEvent()}
		<section class="gifting hero-content">
			<div class="content">
				<h1>X-MAS SUB EVENT</h1>
				<p>
					Gift 1 sub to anyone this christmas and get a special badge!
					<br />
					<a href="/store">Gift a sub</a>
				</p>
			</div>
		</section>
	{/if}

	{#if moment("2024-12-20T16:00:00Z").isAfter()}
		<section class="hero nnys dark-theme">
			<div class="hero-content">
				<Logo size={8.5 * 16} />
				<div class="content">
					<h1>
						The Emote Platform <span class="for-all">for All</span>
					</h1>
					<p>
						Manage hundreds of emotes for your Twitch, Kick or YouTube channels with ease. Enhance
						your chatting experience.
					</p>
					<p>Vote on NNYS 2024 to get an exclusive paint, badge & emote set.</p>
				</div>
				<div class="buttons">
					<Button primary style="font-size: 1em;" href="https://www.nnys.live/">Vote now!</Button>
					<Button style="font-size: 1em;" href="#download">Download</Button>
				</div>
			</div>
			<img class="hero-image hide-on-mobile hero-floating" src={nnysPlaceholder} alt="7TV" />
		</section>
	{:else if moment("2025-03-02T00:00:00Z").isAfter()}
		<section class="hero pickems dark-theme">
			<div class="hero-content">
				<Logo size={8.5 * 16} />
				<div class="content">
					<h1>
						The Emote Platform <span class="for-all">for All</span>
					</h1>
					<p>Place your Pick'ems. Win Prizes.</p>
					<p>7TV Hoster CS2 Tournament • Feb. 29th - Mar. 2nd</p>
				</div>
				<div class="buttons">
					{#if hasPass}
						<Button primary style="font-size: 1em;" href="https://app.pickems.tv">
							{#snippet iconRight()}
								<ArrowSquareOut />
							{/snippet}
							Place your pick'ems
						</Button>
					{:else}
						<Button primary style="font-size: 1em;" href="/store">
							{#snippet iconRight()}
								<Ticket />
							{/snippet}
							Get the pass!
						</Button>
						<Button style="font-size: 1em;" href="https://app.pickems.tv">
							{#snippet iconRight()}
								<ArrowSquareOut />
							{/snippet}
							Learn More
						</Button>
					{/if}
				</div>
			</div>
			<!-- <img class="hero-image hide-on-mobile" src={pickemsBanner} alt="7TV" /> -->
		</section>
	{:else}
		<section class="hero">
			<div class="hero-content">
				<Logo size={8.5 * 16} />
				<div class="content">
					<h1>
						The Emote Platform <span class="for-all">for All</span>
					</h1>
					<p>
						Manage hundreds of emotes for your Twitch, Kick or YouTube channels with ease. Enhance
						your chatting experience.
					</p>
				</div>
				<div class="buttons">
					<Button primary style="font-size: 1em;" href="#download">Download</Button>
					<Button style="font-size: 1em;">Learn More</Button>
				</div>
			</div>
			<img class="hero-image hide-on-mobile hero-floating" src={landingPagePlaceholder} alt="7TV" />
		</section>
	{/if}
</div>

<style lang="scss">
	@keyframes backgroundScroll {
		0% {
			background-position: 0 0;
		}
		100% {
			background-position: 0 -40rem;
		}
	}

	@keyframes heroFloat {
		0% {
			transform: translate3d(0, 0px, 0) rotate(0deg);
			animation-timing-function: cubic-bezier(0.445, 0.05, 0.55, 0.95);
		}
		50% {
			transform: translate3d(0, -20px, 0) rotate(1deg);
			animation-timing-function: cubic-bezier(0.445, 0.05, 0.55, 0.95);
		}
		100% {
			transform: translate3d(0, 0px, 0) rotate(0deg);
		}
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

		&.pickems {
			overflow: clip;
			border-radius: 1rem;
			border: solid 1px hsla(0deg, 0%, 100%, 20%);
			&::before {
				content: "";
				position: absolute;
				inset: 0 0 0 0;
				background: radial-gradient(10rem circle at 10rem 0, purple, transparent),
					radial-gradient(10rem circle at 10rem 20rem, pink, transparent),
					radial-gradient(10rem circle at 10rem 40rem, purple, transparent),
					radial-gradient(10rem circle at 30rem 10rem, lightblue, transparent),
					radial-gradient(10rem circle at 30rem 30rem, blue, transparent),
					radial-gradient(10rem circle at 50rem 0, yellow, transparent),
					radial-gradient(10rem circle at 50rem 20rem, red, transparent),
					radial-gradient(10rem circle at 50rem 40rem, yellow, transparent),
					radial-gradient(10rem circle at 70rem 10rem, blue, transparent),
					radial-gradient(10rem circle at 70rem 30rem, orange, transparent);
				background-size: 80rem 40rem;
				filter: blur(4rem);
				mask-size: unset;
				mask-image: unset;

				animation: backgroundScroll 20s linear infinite;
			}
		}

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
		max-width: 25rem;

		margin-inline: auto;
		
		/* Floating animation */
		animation: heroFloat 6s ease-in-out infinite !important;
		transform-origin: center center !important;
		will-change: transform;
		backface-visibility: hidden;
		transform: translateZ(0);
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
