<script lang="ts">
	import { user } from "$/lib/stores";
	import Button from "../input/button.svelte";
	import DiscordLogo from "../icons/discord-logo.svelte";
	import Logo from "../icons/logo.svelte";
	import GoogleLogo from "../icons/google-logo.svelte";
	import TwitchLogo from "../icons/twitch-logo.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import { Envelope } from "phosphor-svelte";
	import { t } from "svelte-i18n";

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
			<h1>{$t("dialogs.sign_in.title")}</h1>
			<span class="details">{$t("dialogs.sign_in.subtitle")}</span>
		</div>
		<div class="buttons">
			<Button secondary big on:click={login}>
				<Envelope slot="icon" />
				{$t("dialogs.sign_in.email")}
			</Button>
			<Button secondary big on:click={login}>
				<TwitchLogo slot="icon" />
				{$t("dialogs.sign_in.continue_with", { values: { platform: "Twitch" } })}
			</Button>
			<Button secondary big on:click={login}>
				<GoogleLogo slot="icon" />
				{$t("dialogs.sign_in.continue_with", { values: { platform: "Google" } })}
			</Button>
			<Button secondary big on:click={login}>
				<DiscordLogo slot="icon" />
				{$t("dialogs.sign_in.continue_with", { values: { platform: "Discord" } })}
			</Button>
			<a class="trouble" href="/trouble" on:click={() => (mode = DialogMode.Hidden)}>
				{$t("dialogs.sign_in.trouble")}
			</a>
		</div>
		<hr />
		<!-- i18n could be improved here -->
		<span class="legal-yapping">
			{$t("dialogs.sign_in.legal_yapping")}
			<a href="/tos" target="_blank">{$t("common.tos")}</a>
			{$t("dialogs.sign_in.and")}
			<a href="/privacy" target="_blank">{$t("common.privacy")}</a>.
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
