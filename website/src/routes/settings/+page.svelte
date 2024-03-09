<script lang="ts">
	import Button from "$/components/button.svelte";
	import XTwitterLogo from "$/components/icons/x-twitter-logo.svelte";
	import TwitchLogo from "$/components/icons/twitch-logo.svelte";
	import YoutubeLogo from "$/components/icons/youtube-logo.svelte";
	import DiscordLogo from "$/components/icons/discord-logo.svelte";
	import Select from "$/components/select.svelte";
	import KickLogo from "$/components/icons/kick-logo.svelte";
	import { At, Trash, Password } from "phosphor-svelte";
	import Toggle from "$/components/toggle.svelte";
	import Checkbox from "$/components/checkbox.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import DeleteAccountDialog from "$/components/dialogs/delete-account-dialog.svelte";

	let twoFaActive = false;
	let deleteAccountDialogMode = DialogMode.Hidden;
</script>

<svelte:head>
	<title>Account Settings - 7TV</title>
</svelte:head>

<DeleteAccountDialog bind:mode={deleteAccountDialogMode} />
<section>
	<div>
		<h2>Profile</h2>
		<span class="details">
			Customize preferences and manage details to suit your individual needs
		</span>
	</div>
	<div class="content">
		<span>
			<h3>Display Name</h3>
			<span class="details">
				Choose a main connection that will be used for your display name and static profile picture
			</span>
		</span>
		<Select
			options={[
				{ value: "twitch", label: "ayyybubu", icon: TwitchLogo },
				{ value: "youtube", label: "ayyybubu", icon: YoutubeLogo },
				{ value: "kick", label: "gambabubu", icon: KickLogo },
			]}
			style="align-self: flex-start"
		/>
		<hr />
		<span>
			<h3>Profile Picture</h3>
			<span class="details">
				Choose a profile picture that will be displayed on all platforms
			</span>
		</span>
		<div class="profile-picture">
			<div class="placeholder"></div>
			<div class="buttons">
				<Button secondary>Update Profile Picture</Button>
				<Button>
					<Trash slot="icon" />
				</Button>
			</div>
			<span class="limits">TODO Max size and frames</span>
		</div>
	</div>
</section>

<section>
	<div>
		<h2>Connections</h2>
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
			<Button secondary>Disconnect</Button>
		</li>
		<li>
			<div class="platform">
				<YoutubeLogo />
				<span>YouTube – ayyybubu</span>
			</div>
			<Button secondary>Disconnect</Button>
		</li>
		<li>
			<div class="platform">
				<KickLogo />
				<span>Kick – gambabubu</span>
			</div>
			<Button secondary>Disconnect</Button>
		</li>
		<li>
			<div class="platform">
				<DiscordLogo />
				<span>Discord</span>
			</div>
			<Button primary>Connect</Button>
		</li>
		<li>
			<div class="platform">
				<XTwitterLogo />
				<span>X / Twitter</span>
			</div>
			<Button primary>Connect</Button>
		</li>
	</ul>
</section>

<section>
	<div>
		<h2>Security</h2>
		<span class="details">
			Manage and customize your account security with options like password updates and two-factor
			authentication
		</span>
	</div>
	<div class="content">
		<TextInput type="email" style="max-width: 30rem">
			<At slot="icon" />
			<h3>Email</h3>
		</TextInput>
		<TextInput type="password" style="max-width: 30rem">
			<Password slot="icon" />
			<h3>Password</h3>
		</TextInput>
		<hr />
		<Toggle bind:value={twoFaActive}>
			<div>
				<h3>Two Factor Authentication</h3>
				<span class="details"
					>Enhance the security of your account with Two-Factor Authentication (2FA)</span
				>
			</div>
		</Toggle>
		{#if twoFaActive}
			<Checkbox>
				<div>
					<h3>Email</h3>
					<span class="details">Receive a verification code via email</span>
				</div>
			</Checkbox>
			<Checkbox>
				<div>
					<h3>Authenticator App</h3>
					<span class="details">Install an app to generate your verification code</span>
				</div>
			</Checkbox>
		{/if}
		<hr />
		<span>
			<h3>Sign Out Everywhere</h3>
			<span class="details">Ensure security and log out from all devices with a single click</span>
		</span>
		<Button secondary style="align-self: flex-start">Sign Out Everywhere</Button>
		<hr />
		<span>
			<h3>Delete Account</h3>
			<span class="details"
				>Permanently remove all personal information and associated data from the platform,
				terminating your account</span
			>
		</span>
		<Button
			secondary
			style="align-self: flex-start; color: var(--error);"
			on:click={() => (deleteAccountDialogMode = DialogMode.Shown)}>Delete Account</Button
		>
	</div>
</section>

<style lang="scss">
	@import "../../styles/settings.scss";

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
