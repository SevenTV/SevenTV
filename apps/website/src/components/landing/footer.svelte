<script lang="ts">
	import DiscordLogo from "../icons/discord-logo.svelte";
	import GithubLogo from "../icons/github-logo.svelte";
	import Logo from "../icons/logo.svelte";
	import XTwitterLogo from "../icons/x-twitter-logo.svelte";
	import {
		PUBLIC_TWITTER_LINK,
		PUBLIC_GITHUB_LINK,
		PUBLIC_DISCORD_LINK,
		PUBLIC_DEVELOPER_PORTAL,
	} from "$env/static/public";

	import buh from "$assets/buh.webp?url";

	let buhing = $state(false);
</script>

<footer>
	<div class="row">
		<div class="logo-container" onclick={() => (buhing = !buhing)} role="none">
			{#if buhing}
				<img src={buh} alt="7TV" width="32" height="32" />
			{:else}
				<Logo size={2 * 16} />
			{/if}
		</div>
		<div class="links">
			<a href="/faq">FAQ</a>
			<a href={PUBLIC_DEVELOPER_PORTAL}>Developer Portal</a>
			<a href="/contact">Contact</a>
		</div>
		<div class="socials">
			<a href={PUBLIC_TWITTER_LINK}><XTwitterLogo size={1.6 * 16} /></a>
			<a href={PUBLIC_GITHUB_LINK}><GithubLogo size={1.6 * 16} /></a>
			<a href={PUBLIC_DISCORD_LINK}><DiscordLogo size={1.6 * 16} /></a>
		</div>
	</div>
	<hr />
	<div class="links small">
		<span>7TV is not affiliated with Twitch, Kick, YouTube or Discord</span>
		<a href="/privacy">Privacy Policy</a>
		<a href="/tos">Terms of Service</a>
	</div>
</footer>

<style lang="scss">
	footer {
		display: flex;
		flex-direction: column;
		gap: 0.8rem;

		text-align: center;
		padding: 2rem;

		.row {
			display: flex;
			justify-content: space-between;
			gap: 1rem 2rem;
			flex-wrap: wrap-reverse;
		}

		.links {
			display: flex;
			gap: 1rem 2rem;
			justify-content: center;
			align-items: center;
			flex-wrap: wrap;

			font-size: 1rem;
			font-weight: 600;

			&.small {
				font-size: 0.875rem;
				font-weight: 400;

				a {
					color: var(--store);
				}
			}
		}

		.socials {
			flex-grow: 1;

			display: flex;
			gap: 1rem;
			justify-content: flex-end;

			& > *:hover {
				color: var(--store);
			}
		}

		.logo-container {
			flex-grow: 1;

			display: flex;
			align-items: center;
			justify-content: flex-start;
		}

		a {
			color: var(--text);
		}
	}

	@media screen and (max-width: 960px) {
		footer {
			padding: 1rem;

			.links {
				gap: 0.5rem 1rem;
			}
		}
	}
</style>
