<script lang="ts">
	import { user } from "$/lib/stores";
	import Button from "../input/button.svelte";
	import DiscordLogo from "../icons/discord-logo.svelte";
	import Logo from "../icons/logo.svelte";
	import GoogleLogo from "../icons/google-logo.svelte";
	import TwitchLogo from "../icons/twitch-logo.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import { Envelope } from "phosphor-svelte";

	export let mode: DialogMode = DialogMode.Hidden;

	function login() {
		$user = true;
		mode = DialogMode.Hidden;
	}
</script>

<Dialog bind:mode>
	<div class="layout">
		<div class="header">
			<Logo size={3 * 16} />
			<h1>Sign in to 7TV</h1>
			<span class="details">Sign in or create an account to continue</span>
		</div>
		<div class="buttons">
			<Button secondary big on:click={login}>
				<Envelope slot="icon" />
				Use Email
			</Button>
			<Button secondary big on:click={login}>
				<TwitchLogo slot="icon" />
				Continue with Twitch
			</Button>
			<Button secondary big on:click={login}>
				<GoogleLogo slot="icon" />
				Continue with Google
			</Button>
			<Button secondary big on:click={login}>
				<DiscordLogo slot="icon" />
				Continue with Discord
			</Button>
			<a class="trouble" href="/trouble" on:click={() => (mode = DialogMode.Hidden)}
				>Trouble signing in?</a
			>
		</div>
		<hr />
		<span class="legal-yapping">
			By continuing, you agree to the <a href="/tos" target="_blank">Terms of Service</a> and the
			<a href="/privacy" target="_blank">Privacy Policy</a>.
		</span>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		margin-block: auto;
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.header {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;

		h1 {
			font-size: 1.5rem;
			font-weight: 600;
		}

		.details {
			color: var(--text-light);
		}
	}

	.buttons {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		& > :global(.button) {
			justify-content: center;
			background-color: var(--bg-light);
		}

		.trouble {
			color: var(--text-light);
			font-size: 0.75rem;
			font-weight: 500;
			text-align: center;
		}
	}

	.legal-yapping {
		color: var(--text-light);
		font-size: 0.75rem;
		font-weight: 500;

		a {
			color: var(--primary);
		}
	}
</style>
