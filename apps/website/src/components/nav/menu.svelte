<script lang="ts">
	import { theme, type Theme } from "$/lib/layout";
	import { logout, user } from "$/lib/auth";
	import Role from "../users/role.svelte";
	import { fade } from "svelte/transition";
	import {
		Bell,
		CaretLeft,
		CaretRight,
		CreditCard,
		GlobeHemisphereWest,
		House,
		Key,
		Moon,
		PaintBrush,
		PencilSimple,
		Prohibit,
		SignOut,
		Sliders,
		Smiley,
		Star,
		Sun,
	} from "phosphor-svelte";
	import MenuButton from "../input/menu-button.svelte";
	import { locale, dictionary, t } from "svelte-i18n";
	import { localeNames } from "$/lib/i18n";
	import UserProfilePicture from "../user-profile-picture.svelte";
	import Spinner from "../spinner.svelte";
	import { filterRoles } from "$/lib/utils";
	import UserName from "../user-name.svelte";

	let { onCloseRequest }: { onCloseRequest?: () => void } = $props();

	type Menu = "root" | "language" | "theme" | "settings";

	let menu: Menu = $state("root");

	function setMenu(e: MouseEvent, newMenu: Menu) {
		menu = newMenu;
		e.stopPropagation();
	}

	function setTheme(newTheme: Theme) {
		$theme = newTheme;
		onCloseRequest?.();
	}

	let logoutLoading = $state(false);

	function logoutClick() {
		logoutLoading = true;
		logout().then(() => {
			logoutLoading = false;
			onCloseRequest?.();
		});
	}

	let reversedRoles = $derived(filterRoles($user?.roles || []));
</script>

<nav class="menu" transition:fade={{ duration: 100 }}>
	{#if menu === "root"}
		{#if $user}
			<a class="profile" href="/users/{$user.id}" onclick={onCloseRequest}>
				<UserProfilePicture user={$user} size={3 * 16} style="grid-row: 1 / -1" />
				<span class="name">
					<UserName user={$user} />
				</span>
				<div class="roles">
					{#each reversedRoles as role}
						<Role {role} />
					{/each}
				</div>
				<div class="caret">
					<CaretRight size={1.2 * 16} />
				</div>
			</a>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list hide-on-desktop">
			<MenuButton href="/">
				<House />
				{$t("pages.home.title")}
			</MenuButton>
			<MenuButton href="/emotes">
				<Smiley />
				{$t("common.emotes", { values: { count: 2 } })}
			</MenuButton>
			<!-- <MenuButton href="/discover">
				<Compass />
				{$t("pages.discover.title")}
			</MenuButton> -->
			<MenuButton href="/store" style="color: var(--store)">
				<Star />
				{$t("pages.store.title")}
			</MenuButton>
		</div>
		{#if $user}
			<div class="link-list">
				<MenuButton href="/cosmetics" onclick={onCloseRequest}>
					<PaintBrush />
					{$t("common.cosmetics")}
				</MenuButton>
				<!-- <MenuButton href="/analytics">
					<ChartLine />
					{$t("common.analytics")}
				</MenuButton> -->
			</div>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list">
			<!-- <MenuButton showCaret on:click={(e) => setMenu(e, Menu.Language)}>
				<GlobeHemisphereWest />
				{$t("common.language")}
			</MenuButton> -->
			<MenuButton showCaret onclick={(e) => setMenu(e, "theme")}>
				{#if $theme === "system-theme"}
					<Sliders />
				{:else if $theme === "light-theme"}
					<Sun />
				{:else}
					<Moon />
				{/if}
				{$t("common.theme")}
			</MenuButton>
			<!-- {#if $user}
				<MenuButton href="/settings" hideOnMobile>
					<Gear />
					{$t("common.settings")}
				</MenuButton>
				<MenuButton showCaret hideOnDesktop on:click={(e) => setMenu(e, Menu.Settings)}>
					<Gear />
					{$t("common.settings")}
				</MenuButton>
			{/if} -->
		</div>
		<hr class="hide-on-mobile" />
		<div class="link-list">
			<!-- <MenuButton href={PUBLIC_DEVELOPER_PORTAL}>
				<Code />
				{$t("common.developer_portal")}
			</MenuButton> -->
			<!-- <MenuButton href="/contact">
				<ChatDots />
				{$t("common.contact")}
			</MenuButton>
			<MenuButton href="/faq">
				<Question />
				{$t("common.faq_short")}
			</MenuButton>
			<MenuButton href="/privacy">
				<LockSimple />
				{$t("common.privacy")}
			</MenuButton>
			<MenuButton href="/tos">
				<Note />
				{$t("common.tos")}
			</MenuButton> -->
		</div>
		{#if $user}
			<!-- <hr class="hide-on-mobile" /> -->
			<div class="link-list">
				<MenuButton onclick={logoutClick}>
					{#if logoutLoading}
						<Spinner size={1.2 * 16} />
					{:else}
						<SignOut />
					{/if}
					{$t("common.sign_out")}
				</MenuButton>
			</div>
		{/if}
	{:else if menu === "language"}
		<MenuButton onclick={() => (menu = "root")}>
			<CaretLeft />
			{$t("common.language")}
		</MenuButton>
		<div class="link-list">
			{#each Object.keys($dictionary) as l}
				<MenuButton onclick={() => ($locale = l)}>
					<GlobeHemisphereWest />
					{localeNames[l] || l}
				</MenuButton>
			{/each}
		</div>
	{:else if menu === "theme"}
		<MenuButton onclick={() => (menu = "root")}>
			<CaretLeft />
			{$t("common.theme")}
		</MenuButton>
		<div class="link-list">
			<MenuButton onclick={() => setTheme("system-theme")}>
				<Sliders />
				{$t("themes.system")}
			</MenuButton>
			<MenuButton onclick={() => setTheme("dark-theme")}>
				<Moon />
				{$t("themes.dark")}
			</MenuButton>
			<MenuButton onclick={() => setTheme("light-theme")}>
				<Sun />
				{$t("themes.light")}
			</MenuButton>
		</div>
	{:else if menu === "settings"}
		<MenuButton onclick={() => (menu = "root")}>
			<CaretLeft />
			{$t("common.settings")}
		</MenuButton>
		<div class="link-list">
			<MenuButton href="/settings/account">
				<Key />
				{$t("pages.settings.account.title")}
			</MenuButton>
			<MenuButton href="/settings/editors">
				<PencilSimple />
				{$t("common.editors")}
			</MenuButton>
		</div>
		<div class="link-list">
			<MenuButton href="/settings/notifications">
				<Bell />
				{$t("common.notifications")}
			</MenuButton>
			<MenuButton href="/settings/blocked">
				<Prohibit />
				{$t("pages.settings.blocked.title")}
			</MenuButton>
			<MenuButton href="/settings/billing">
				<CreditCard />
				{$t("pages.settings.billing.title")}
			</MenuButton>
		</div>
	{/if}
</nav>

<style lang="scss">
	.menu {
		display: flex;
		flex-direction: column;

		text-align: left;

		min-width: 14rem;
	}

	.profile {
		color: var(--text);
		text-decoration: none;
		padding: 1rem;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		display: grid;
		grid-template-columns: auto 1fr auto;
		grid-template-rows: auto auto;
		align-items: center;
		row-gap: 0.5rem;
		column-gap: 0.75rem;

		.name {
			grid-row: 1;
			font-size: 1rem;
			font-weight: 600;
		}

		.roles {
			grid-row: 2;

			display: flex;
			flex-wrap: wrap;
			gap: 0.25rem;
		}

		.caret {
			grid-row: 1 / -1;
			justify-self: end;

			color: var(--text);
		}

		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}
	}

	.link-list {
		display: flex;
		flex-direction: column;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}

	@media screen and (max-width: 960px) {
		.menu {
			padding: 0.5rem 1rem;
			gap: 0.5rem;
		}
	}
</style>
