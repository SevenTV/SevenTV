<script lang="ts">
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
	import landingPagePlaceholder from "$assets/landing-page-placeholder.webp?url";
	import nnysPlaceholder from "$assets/nnys.webp?url";
	import { isXmasEvent } from "$/lib/xmas";
	import moment from "moment";
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
			<img class="hero-image hide-on-mobile" src={nnysPlaceholder} alt="7TV" />
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
			<img class="hero-image hide-on-mobile" src={landingPagePlaceholder} alt="7TV" />
		</section>
	{/if}
</div>

<style lang="scss">
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
