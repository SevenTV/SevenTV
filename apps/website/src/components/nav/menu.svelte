<script lang="ts">
	import { theme, type Theme } from "$/lib/layout";
	import { logout, user, pendingEditorFor } from "$/lib/auth";
	import Role from "../users/role.svelte";
	import { fade } from "svelte/transition";
	import {
		CaretLeft,
		CaretRight,
		Check,
		Gear,
		GlobeHemisphereWest,
		House,
		LockSimple,
		Moon,
		Note,
		PaintBrush,
		SignOut,
		Sliders,
		Smiley,
		Star,
		Sun,
		Wrench,
	} from "phosphor-svelte";
	import MenuButton from "../input/menu-button.svelte";
	import { locale, dictionary, t } from "svelte-i18n";
	import { localeNames } from "$/lib/i18n";
	import UserProfilePicture from "../user-profile-picture.svelte";
	import Spinner from "../spinner.svelte";
	import { filterRoles } from "$/lib/utils";
	import UserName from "../user-name.svelte";
	import NumberBadge from "../number-badge.svelte";

	let { onCloseRequest }: { onCloseRequest?: () => void } = $props();

	type Menu = "root" | "language" | "theme";

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
		logout()
			.then(() => {
				onCloseRequest?.();
			})
			.finally(() => {
				logoutLoading = false;
			});
	}

	let highestRole = $derived(filterRoles($user?.roles || [])[0]);
</script>

{#snippet caret()}
	<CaretRight />
{/snippet}
{#snippet check()}
	<Check />
{/snippet}

<nav class="menu" transition:fade={{ duration: 100 }}>
	{#if menu === "root"}
		{#if $user}
			<a class="profile" href="/users/{$user.id}" onclick={onCloseRequest}>
				<UserProfilePicture user={$user} size={3 * 16} style="grid-row: 1 / -1" />
				<span class="name">
					<UserName user={$user} />
				</span>
				<div class="role">
					{#if highestRole}
						<Role roleData={highestRole} />
					{/if}
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
			<MenuButton href="/store" style="color: var(--store)">
				<Star />
				{$t("pages.store.title")}
			</MenuButton>
			{#if $user?.permissions.user.manageAny || $user?.permissions.user.manageBilling}
				<MenuButton href="/admin" style="color: var(--staff)">
					<Wrench />
					{$t("pages.admin.title")}
				</MenuButton>
			{/if}
		</div>
		{#if $user}
			<div class="link-list">
				<MenuButton href="/users/{$user.id}/cosmetics" onclick={onCloseRequest}>
					<PaintBrush />
					{$t("common.cosmetics")}
				</MenuButton>
			</div>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list">
			<!-- <MenuButton showCaret on:click={(e) => setMenu(e, Menu.Language)}>
				<GlobeHemisphereWest />
				{$t("common.language")}
			</MenuButton> -->
			<MenuButton iconRight={caret} onclick={(e) => setMenu(e, "theme")}>
				{#if $theme === "system-theme"}
					<Sliders />
				{:else if $theme === "light-theme"}
					<Sun />
				{:else}
					<Moon />
				{/if}
				{$t("common.theme")}
			</MenuButton>
			<MenuButton href="/settings" onclick={onCloseRequest}>
				<Gear />
				{$t("common.settings")}
				<NumberBadge count={$pendingEditorFor} />
			</MenuButton>
		</div>
		<hr class="hide-on-mobile" />
		<div class="link-list">
			<MenuButton href="/privacy">
				<LockSimple />
				{$t("common.privacy")}
			</MenuButton>
			<MenuButton href="/tos">
				<Note />
				{$t("common.tos")}
			</MenuButton>
		</div>
		{#if $user}
			<hr class="hide-on-mobile" />
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
			<MenuButton
				onclick={() => setTheme("system-theme")}
				iconRight={$theme === "system-theme" ? check : undefined}
			>
				<Sliders />
				{$t("themes.system")}
			</MenuButton>
			<MenuButton
				onclick={() => setTheme("dark-theme")}
				iconRight={$theme === "dark-theme" ? check : undefined}
			>
				<Moon />
				{$t("themes.dark")}
			</MenuButton>
			<MenuButton
				onclick={() => setTheme("light-theme")}
				iconRight={$theme === "light-theme" ? check : undefined}
			>
				<Sun />
				{$t("themes.light")}
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
		grid-template-columns: auto 1fr;
		grid-template-rows: auto auto;
		align-items: center;
		row-gap: 0.5rem;
		column-gap: 0.75rem;

		.name {
			grid-row: 1;
			font-size: 1rem;
			font-weight: 600;
		}

		.role {
			grid-row: 2;
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
