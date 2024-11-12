<script lang="ts">
	import Button from "../input/button.svelte";
	import DiscordLogo from "../icons/discord-logo.svelte";
	import Logo from "../icons/logo.svelte";
	import TwitchLogo from "../icons/twitch-logo.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import { PUBLIC_REST_API_V4 } from "$env/static/public";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();
</script>

<Dialog bind:mode>
	<div class="layout">
		<div class="header">
			<Logo size={3 * 16} />
			<h1>{$t("dialogs.sign_in.title")}</h1>
			<span class="details">{$t("dialogs.sign_in.subtitle")}</span>
		</div>
		<div class="buttons">
			<Button secondary big href="{PUBLIC_REST_API_V4}/auth/login?platform=twitch">
				{#snippet icon()}
					<TwitchLogo />
				{/snippet}
				{$t("dialogs.sign_in.continue_with", { values: { platform: "Twitch" } })}
			</Button>
			<Button secondary big href="{PUBLIC_REST_API_V4}/auth/login?platform=discord">
				{#snippet icon()}
					<DiscordLogo />
				{/snippet}
				{$t("dialogs.sign_in.continue_with", { values: { platform: "Discord" } })}
			</Button>
			<!-- <Button secondary big href="{PUBLIC_REST_API_V4}/auth/login?platform=google">
				<GoogleLogo />
				{$t("dialogs.sign_in.continue_with", { values: { platform: "Google" } })}
			</Button> -->
			<a class="trouble" href="/trouble" onclick={() => (mode = "hidden")}>
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
