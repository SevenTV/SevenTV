<script lang="ts">
	import Logo from "$/components/icons/logo.svelte";
	import TopTabs from "./top-tabs.svelte";
	import Badge from "../badge.svelte";
	import HideOn from "../hide-on.svelte";
	import { showMobileMenu, uploadDialogMode } from "$/store/layout";
	import { user } from "$/store/auth";
	import DropDown from "../drop-down.svelte";
	import Menu from "./menu.svelte";
	import DirectMessages from "../direct-messages/direct-messages.svelte";
	import Notifications from "../notifications/notifications.svelte";
	import {
		Bell,
		Chat,
		List,
		MagnifyingGlass,
		PlusSquare,
		ShoppingCartSimple,
	} from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import CartDialog from "../dialogs/cart-dialog.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { DialogMode } from "../dialogs/dialog.svelte";
	import { t } from "svelte-i18n";

	let cartDialogMode = DialogMode.Hidden;
</script>

<CartDialog bind:mode={cartDialogMode} />
<nav>
	<div class="links">
		<a class="home" href="/">
			<Logo />
		</a>
		<HideOn mobile>
			<TopTabs
				tabs={[
					{ name: $t("common.emotes", { values: { count: 2 } }), pathname: "/emotes" },
					// { name: $t("pages.discover.title"), pathname: "/discover" },
					// { name: $t("pages.store.title"), pathname: "/store", highlight: "var(--store)" },
					// { name: $t("pages.admin.title"), pathname: "/admin", highlight: "var(--staff)" },
				]}
			/>
		</HideOn>
	</div>
	<!-- <HideOn mobile>
		<TextInput placeholder={$t("labels.search")} big style="flex: 0 1 20rem">
			<MagnifyingGlass slot="icon" />
		</TextInput>
	</HideOn> -->
	<div class="user-actions">
		<!-- <Button hideOnDesktop>
			<MagnifyingGlass slot="icon" />
		</Button> -->
		{#if $user}
			<DropDown hideOnMobile>
				<Button>
					<Badge count={0} slot="icon">
						<Bell />
					</Badge>
				</Button>
				<Notifications popup slot="dropdown" />
			</DropDown>
			<Button href="/notifications" hideOnDesktop>
				<Badge count={0} slot="icon">
					<Bell />
				</Badge>
			</Button>

			<DropDown hideOnMobile>
				<Button>
					<Badge count={1} slot="icon">
						<Chat />
					</Badge>
				</Button>
				<DirectMessages popup slot="dropdown" />
			</DropDown>
			<Button href="/direct-messages" hideOnDesktop>
				<Badge count={1} slot="icon">
					<Chat />
				</Badge>
			</Button>

			<Button on:click={() => (cartDialogMode = DialogMode.Shown)}>
				<Badge count={3} slot="icon">
					<ShoppingCartSimple />
				</Badge>
			</Button>

			<Button hideOnDesktop on:click={() => ($uploadDialogMode = DialogMode.Shown)}>
				<PlusSquare slot="icon" />
			</Button>
			<Button secondary hideOnMobile on:click={() => ($uploadDialogMode = DialogMode.Shown)}>
				<PlusSquare slot="icon" />
				{$t("dialogs.upload.upload")}
			</Button>
			<HideOn mobile>
				<DropDown>
					<img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
					<span class="profile-name">ayyybubu</span>
					<Menu slot="dropdown" />
				</DropDown>
			</HideOn>
			<button class="profile hide-on-desktop" on:click={() => ($showMobileMenu = !$showMobileMenu)}>
				<img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
			</button>
		{:else}
			<HideOn mobile>
				<DropDown>
					<Button>
						<List slot="icon" />
					</Button>
					<Menu slot="dropdown" />
				</DropDown>
			</HideOn>
			<!-- <Button primary on:click={() => ($signInDialogMode = DialogMode.Shown)}>
				{$t("common.sign_in")}
			</Button> -->
		{/if}
		<!-- Only show when logged out on mobile -->
		{#if !$user}
			<Button hideOnDesktop on:click={() => ($showMobileMenu = !$showMobileMenu)}>
				<List slot="icon" />
			</Button>
		{/if}
	</div>
</nav>

<style lang="scss">
	nav {
		background-color: var(--bg-dark);
		border-bottom: 1px solid var(--border-active);
		padding: 0 2rem;
		height: 3.5rem;

		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
	}

	.links {
		/* Take all available space but shrink by a very high factor */
		flex: 1 9999;

		display: flex;
		gap: 2rem;

		.home {
			color: var(--text);

			display: flex;
			align-items: center;
		}
	}

	.user-actions {
		/* Take all available space but shrink by a very high factor */
		flex: 1 9999;

		display: flex;
		gap: 0.75rem;
		align-items: center;
		justify-content: flex-end;

		.profile {
			color: var(--text);

			display: flex;
			align-items: center;
			gap: 0.5rem;
			text-decoration: none;
		}

		.profile-name {
			font-weight: 600;
			color: var(--staff);
		}

		.profile-picture {
			width: 2rem;
			height: 2rem;

			border-radius: 50%;
			border: 2px solid var(--staff);
		}
	}

	@media screen and (max-width: 960px) {
		nav {
			padding: 0 1rem;
			gap: 1rem;
		}

		.links {
			gap: 1rem;
		}
	}
</style>
