<script lang="ts">
	import Logo from "../icons/logo.svelte";
	import Button from "../input/button.svelte";
</script>

<section class="hero">
	<div class="hero-content">
		<Logo size={8.5 * 16} />
		<div class="content">
			<h1>
				The Emote Platform <span class="for-all">for All</span>
			</h1>
			<p>
				Manage hundreds of emotes for your Twitch, Kick or YouTube channels with ease. Enhance your
				chatting experience.
			</p>
		</div>
		<div class="buttons">
			<Button primary style="font-size: 1em;" href="#download">Download</Button>
			<Button style="font-size: 1em;" href="#learn-more">Learn More</Button>
		</div>
	</div>
	<img class="hero-image hide-on-mobile" src="/landing-page-placeholder.png" alt="7TV" />
</section>

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

		perspective: 1000px;

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

			// mask-image: linear-gradient(transparent, white);
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
			gap: 1rem;
		}
	}

	.hero-image {
		width: 100%;
		min-width: 15rem;
		max-width: 25rem;

		margin-inline: auto;
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
