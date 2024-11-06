<script lang="ts">
	import Logo from "$/components/icons/logo.svelte";
	import TopTabs from "./top-tabs.svelte";
	import HideOn from "../hide-on.svelte";
	import { showMobileMenu, uploadDialogMode, signInDialogMode } from "$/lib/layout";
	import { user } from "$/lib/auth";
	import DropDown from "../drop-down.svelte";
	import Menu from "./menu.svelte";
	import { List, PlusSquare } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import CartDialog from "../dialogs/cart-dialog.svelte";
	import { type DialogMode } from "../dialogs/dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";
	import UserProfilePicture from "../user-profile-picture.svelte";

	let cartDialogMode: DialogMode = $state("hidden");
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
			<MagnifyingGlass />
		</TextInput>
	</HideOn> -->
	<div class="user-actions">
		<!-- <Button hideOnDesktop>
			<MagnifyingGlass />
		</Button> -->
		{#if $user}
			<!-- <DropDown hideOnMobile>
				<Button>
					<Badge count={0} slot="icon">
						<Bell />
					</Badge>
				</Button>
				<Notifications popup />
			</DropDown>
			<Button href="/notifications" hideOnDesktop>
				<Badge count={0} slot="icon">
					<Bell />
				</Badge>
			</Button> -->

			<!-- <DropDown hideOnMobile>
				<Button>
					<Badge count={1} slot="icon">
						<Chat />
					</Badge>
				</Button>
				<DirectMessages popup />
			</DropDown>
			<Button href="/direct-messages" hideOnDesktop>
				<Badge count={1} slot="icon">
					<Chat />
				</Badge>
			</Button> -->

			<!-- <Button on:click={() => (cartDialogMode = "shown")}>
				<Badge count={3} slot="icon">
					<ShoppingCartSimple />
				</Badge>
			</Button> -->

			<Button hideOnDesktop onclick={() => ($uploadDialogMode = "shown")}>
				{#snippet icon()}
					<PlusSquare />
				{/snippet}
			</Button>
			<Button secondary hideOnMobile onclick={() => ($uploadDialogMode = "shown")}>
				{#snippet icon()}
					<PlusSquare />
				{/snippet}
				{$t("dialogs.upload.upload")}
			</Button>
		{:else if $user === undefined}
			<Spinner />
		{/if}

		{#if $user !== undefined}
			<HideOn mobile>
				<DropDown>
					{#if $user}
						<UserProfilePicture user={$user} size={32} />
						<span class="profile-name">{$user.mainConnection?.platformDisplayName}</span>
					{:else}
						<Button>
							{#snippet icon()}
								<List />
							{/snippet}
						</Button>
					{/if}
					{#snippet dropdown(close)}
						<Menu onCloseRequest={close} />
					{/snippet}
				</DropDown>
			</HideOn>
		{/if}

		{#if $user}
			<button class="profile hide-on-desktop" onclick={() => ($showMobileMenu = !$showMobileMenu)}>
				<UserProfilePicture user={$user} size={32} />
			</button>
		{:else if $user === null}
			<Button primary onclick={() => ($signInDialogMode = "shown")}>
				{$t("common.sign_in")}
			</Button>
		{/if}
		<!-- Only show when logged out on mobile -->
		{#if !$user}
			<Button hideOnDesktop onclick={() => ($showMobileMenu = !$showMobileMenu)}>
				{#snippet icon()}
					<List />
				{/snippet}
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
