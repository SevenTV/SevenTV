<script lang="ts">
	import DiscordLogo from "../icons/discord-logo.svelte";
	import GithubLogo from "../icons/github-logo.svelte";
	import Logo from "../icons/logo.svelte";
	import XTwitterLogo from "../icons/x-twitter-logo.svelte";

    let buhing = false;
</script>

<footer>
    <div class="row">
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <div class="logo-container" on:click={() => (buhing = !buhing)} role="none">
            {#if buhing}
                <img src="/buh.webp" alt="7TV" width="32" height="32" />
            {:else}
                <Logo size={2 * 16} />
            {/if}
        </div>
        <div class="links">
            <a href="/faq">FAQ</a>
            <a href="https://7tv.io/">Developer Portal</a>
            <a href="/contact">Contact</a>
        </div>
        <div class="socials">
            <a href="https://x.com/Official_7TV"><XTwitterLogo size="1.6rem" /></a>
            <a href="https://github.com/SevenTV"><GithubLogo size="1.6rem" /></a>
            <a href="https://discord.gg/7tv"><DiscordLogo size="1.6rem" /></a>
        </div>
    </div>
    <hr />
    <div class="links small">
        <span>7TV is not affiliated with Twitch, Kick or YouTube</span>
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
