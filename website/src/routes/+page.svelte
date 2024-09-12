<script lang="ts">
	import Logo from "$/components/icons/logo.svelte";
	import Button from "$/components/input/button.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.suffix")}</title>
</svelte:head>

<div class="layout">
	<section class="hero">
		<div class="hero-content">
			<Logo size={8.5 * 16} />
			<div class="content">
				<h1>
					The&nbsp;Emote Platform <span class="for-all">for All</span>
				</h1>
				<p>
					Manage hundreds of emotes for your Twitch, Kick or YouTube channels with ease. Enhance
					your chatting experience.
				</p>
			</div>
			<div class="buttons">
				<Button primary big href="#">Download</Button>
				<Button big href="#">Learn More</Button>
			</div>
		</div>
		<img class="hero-image" src="/landing-page-placeholder.png" alt="7TV" />
	</section>
	<footer>Footer</footer>
</div>

<style lang="scss">
	// Akony Font
	@font-face {
		font-display: swap;
		font-family: "AKONY";
		font-style: normal;
		font-weight: 400;
		src:
			url("/fonts/akony.woff2") format("woff2"),
			url("/fonts/akony.ttf") format("truetype");
	}

	.layout {
		padding: 1rem 2rem 0rem 2rem;

		display: flex;
		flex-direction: column;
	}

	.hero {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 2rem;
		flex-wrap: wrap;

		padding: 5.5rem;
		min-height: min(50rem, 80vh);
		border-radius: 1rem;

		background: radial-gradient(
				180% 80% at 0% 100%,
				#ffffff 0%,
				#5bf8f9 25%,
				#8a00ffe6 56%,
				#25158480 79%,
				transparent 100%
			),
			#1111110a;
	}

	@media screen and (max-width: 960px) {
		.hero {
			padding: 2rem;
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

				max-width: 27rem;

				font-family: "AKONY", sans-serif;

				.for-all {
					letter-spacing: calc(0.15rem + 2px);
					color: transparent;
					-webkit-text-stroke-width: 2px;
					-webkit-text-stroke-color: white;
				}
			}

			p {
				font-size: 1.125rem;
				font-weight: 400;

				max-width: 37rem;
			}
		}

		.buttons {
			display: flex;
			gap: 1rem;

			// Make better buttons for this page
			--primary: white;
			--primary-text: black;
			--primary-hover: white;
			--primary-active: white;
		}
	}

	.hero-image {
		width: 100%;
		min-width: 15rem;
		max-width: 25rem;

		margin-inline: auto;
	}
</style>
