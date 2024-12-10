<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import XTwitterLogo from "$/components/icons/x-twitter-logo.svelte";
	import TwitchLogo from "$/components/icons/twitch-logo.svelte";
	import YoutubeLogo from "$/components/icons/youtube-logo.svelte";
	import DiscordLogo from "$/components/icons/discord-logo.svelte";
	import Select from "$/components/input/select.svelte";
	import KickLogo from "$/components/icons/kick-logo.svelte";
	import { At, Trash, Password } from "phosphor-svelte";
	import Toggle from "$/components/input/toggle.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { type DialogMode } from "$/components/dialogs/dialog.svelte";
	import DeleteAccountDialog from "$/components/dialogs/delete-account-dialog.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.account_settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#snippet twitchLogo()}
	<TwitchLogo />
{/snippet}
{#snippet youtubeLogo()}
	<YoutubeLogo />
{/snippet}
{#snippet kickLogo()}
	<KickLogo />
{/snippet}

<section>
	<div>
		<h2>{$t("common.profile")}</h2>
		<span class="details">
			{$t("pages.settings.account.profile.details")}
		</span>
	</div>
	<div class="content">
		<span>
			<h3>{$t("pages.settings.account.profile.display_name")}</h3>
			<span class="details">{$t("pages.settings.account.profile.display_name_description")}</span>
		</span>
		<Select
			options={[
				{ value: "twitch", label: "ayyybubu", icon: twitchLogo },
				{ value: "youtube", label: "ayyybubu", icon: youtubeLogo },
				{ value: "kick", label: "gambabubu", icon: kickLogo },
			]}
			style="align-self: flex-start"
		/>
		<hr />
		<span>
			<h3>{$t("common.profile_picture")}</h3>
			<span class="details">{$t("pages.settings.account.profile.profile_picture_description")}</span
			>
		</span>
		<div class="profile-picture">
			<div class="placeholder"></div>
			<div class="buttons">
				<Button secondary>{$t("pages.settings.account.profile.update_profile_picture")}</Button>
				<Button>
					{#snippet icon()}
						<Trash />
					{/snippet}
				</Button>
			</div>
			<span class="limits">
				{$t("file_limits.max_size", { values: { size: "7MB" } })},
				{$t("file_limits.max_resolution", { values: { width: "1000", height: "1000" } })}
			</span>
		</div>
	</div>
</section>

<section>
	<div>
		<h2>{$t("common.connections")}</h2>
		<span class="details">
			Manage and link external accounts to streamline access and enhance interoperability within the
			platform
		</span>
	</div>
	<ul class="content connections">
		<li>
			<div class="platform">
				<TwitchLogo />
				<span>Twitch – ayyybubu</span>
			</div>
			<Button secondary>{$t("labels.disconnect")}</Button>
		</li>
		<li>
			<div class="platform">
				<YoutubeLogo />
				<span>YouTube – ayyybubu</span>
			</div>
			<Button secondary>{$t("labels.disconnect")}</Button>
		</li>
		<li>
			<div class="platform">
				<KickLogo />
				<span>Kick – gambabubu</span>
			</div>
			<Button secondary>{$t("labels.disconnect")}</Button>
		</li>
		<li>
			<div class="platform">
				<DiscordLogo />
				<span>Discord</span>
			</div>
			<Button primary>{$t("labels.connect")}</Button>
		</li>
	</ul>
</section>

<section>
	<div>
		<h2>{$t("pages.settings.account.security.title")}</h2>
		<span class="details">{$t("pages.settings.account.security.details")}</span>
	</div>
	<div class="content">
		<!-- <TextInput type="email" style="max-width: 30rem">
			{#snippet icon()}
				<At />
			{/snippet}
			<h3>{$t("labels.email")}</h3>
		</TextInput>
		<TextInput type="password" style="max-width: 30rem">
			{#snippet icon()}
				<Password />
			{/snippet}
			<h3>{$t("labels.password")}</h3>
		</TextInput>
		<hr />
		<Toggle bind:value={twoFaActive}>
			<div>
				<h3>{$t("pages.settings.account.security.2fa")}</h3>
				<span class="details">{$t("pages.settings.account.security.2fa_details")}</span>
			</div>
		</Toggle>
		{#if twoFaActive}
			<Checkbox>
				<div>
					<h3>{$t("labels.email")}</h3>
					<span class="details">{$t("pages.settings.account.security.email_details")}</span>
				</div>
			</Checkbox>
			<Checkbox>
				<div>
					<h3>{$t("pages.settings.account.security.authenticator_app")}</h3>
					<span class="details">
						{$t("pages.settings.account.security.authenticator_app_details")}
					</span>
				</div>
			</Checkbox>
		{/if}
		<hr /> -->
		<span>
			<h3>{$t("pages.settings.account.security.sign_out_everywhere")}</h3>
			<span class="details">
				{$t("pages.settings.account.security.sign_out_everywhere_details")}
			</span>
		</span>
		<Button secondary style="align-self: flex-start">
			{$t("pages.settings.account.security.sign_out_everywhere")}
		</Button>
		<!-- <hr />
		<span>
			<h3>{$t("common.delete_account")}</h3>
			<span class="details">{$t("pages.settings.account.security.delete_account_details")}</span>
		</span>
		<Button
			secondary
			style="align-self: flex-start; color: var(--danger);"
			onclick={() => (deleteAccountDialogMode = "shown")}
		>
			{$t("common.delete_account")}
		</Button> -->
	</div>
</section>

<style lang="scss">
	@use "../../styles/settings.scss";

	h3 {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.profile-picture {
		display: grid;
		column-gap: 1rem;
		justify-content: start;
		grid-template-columns: repeat(2, auto);

		.placeholder {
			grid-row: 1 / span 2;

			width: 4rem;
			height: 4rem;
			background-color: var(--secondary);
			border-radius: 50%;
		}

		.buttons {
			display: flex;
			align-items: center;
			gap: 0.5rem;
		}

		.limits {
			color: var(--text-light);
			font-size: 0.75rem;
		}
	}

	.connections {
		gap: 0.5rem;

		li {
			display: flex;
			justify-content: space-between;
			align-items: center;

			padding: 0.5rem 1rem;
			background-color: var(--bg-light);
			border-radius: 0.5rem;

			.platform {
				display: flex;
				align-items: center;
				gap: 1rem;

				font-size: 0.875rem;
				font-weight: 500;
			}
		}
	}
</style>
