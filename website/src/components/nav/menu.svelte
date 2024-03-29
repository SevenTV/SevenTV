<script lang="ts">
	import { Theme, theme, user } from "$/lib/stores";
	import Role from "../profile/role.svelte";
	import { fade } from "svelte/transition";
	import {
		Bell,
		CaretLeft,
		CaretRight,
		ChartLine,
		ChatDots,
		Code,
		Compass,
		CreditCard,
		Gear,
		GlobeHemisphereWest,
		House,
		Key,
		LockSimple,
		Moon,
		Note,
		PaintBrush,
		PencilSimple,
		Prohibit,
		Question,
		SealCheck,
		SignOut,
		Sliders,
		Smiley,
		Star,
		Sun,
	} from "phosphor-svelte";
	import MenuButton from "../input/menu-button.svelte";
	import { locale, dictionary, t } from "svelte-i18n";
	import { localeNames } from "$/lib/i18n";

	enum Menu {
		Root,
		Language,
		Theme,
		Settings,
	}

	let menu = Menu.Root;

	function setMenu(e: MouseEvent, newMenu: Menu) {
		menu = newMenu;
		e.stopPropagation();
	}
</script>

<nav class="menu" transition:fade={{ duration: 100 }}>
	{#if menu === Menu.Root}
		{#if $user}
			<a class="profile" href="/user/ayyybubu">
				<img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
				<span class="name">
					ayyybubu
					<SealCheck size="0.8rem" />
				</span>
				<div class="roles">
					<Role name="Staff" />
					<Role name="Subscriber" />
				</div>
				<div class="caret">
					<CaretRight size="1.2rem" />
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
			<MenuButton href="/discover">
				<Compass />
				{$t("pages.discover.title")}
			</MenuButton>
			<MenuButton href="/store" style="color: var(--store)">
				<Star />
				{$t("pages.store.title")}
			</MenuButton>
		</div>
		{#if $user}
			<div class="link-list">
				<MenuButton href="/cosmetics">
					<PaintBrush />
					{$t("common.cosmetics")}
				</MenuButton>
				<MenuButton href="/analytics">
					<ChartLine />
					{$t("common.analytics")}
				</MenuButton>
			</div>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list">
			<MenuButton showCaret on:click={(e) => setMenu(e, Menu.Language)}>
				<GlobeHemisphereWest />
				{$t("common.language")}
			</MenuButton>
			<MenuButton showCaret on:click={(e) => setMenu(e, Menu.Theme)}>
				<Moon />
				{$t("common.theme")}
			</MenuButton>
			{#if $user}
				<MenuButton href="/settings" hideOnMobile>
					<Gear />
					{$t("common.settings")}
				</MenuButton>
				<MenuButton showCaret hideOnDesktop on:click={(e) => setMenu(e, Menu.Settings)}>
					<Gear />
					{$t("common.settings")}
				</MenuButton>
			{/if}
		</div>
		<hr class="hide-on-mobile" />
		<div class="link-list">
			<MenuButton href="https://7tv.io/">
				<Code />
				{$t("common.developer_portal")}
			</MenuButton>
			<MenuButton href="/contact">
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
			</MenuButton>
		</div>
		{#if $user}
			<hr class="hide-on-mobile" />
			<div class="link-list">
				<MenuButton on:click={() => ($user = false)}>
					<SignOut />
					{$t("common.sign_out")}
				</MenuButton>
			</div>
		{/if}
	{:else if menu === Menu.Language}
		<MenuButton on:click={() => (menu = Menu.Root)}>
			<CaretLeft />
			{$t("common.language")}
		</MenuButton>
		<div class="link-list">
			{#each Object.keys($dictionary) as l}
				<MenuButton on:click={() => ($locale = l)}>
					<GlobeHemisphereWest />
					{localeNames[l] || l}
				</MenuButton>
			{/each}
		</div>
	{:else if menu === Menu.Theme}
		<MenuButton on:click={() => (menu = Menu.Root)}>
			<CaretLeft />
			{$t("common.theme")}
		</MenuButton>
		<div class="link-list">
			<MenuButton on:click={() => ($theme = Theme.System)}>
				<Sliders />
				{$t("themes.system")}
			</MenuButton>
			<MenuButton on:click={() => ($theme = Theme.Dark)}>
				<Moon />
				{$t("themes.dark")}
			</MenuButton>
			<MenuButton on:click={() => ($theme = Theme.Light)}>
				<Sun />
				{$t("themes.light")}
			</MenuButton>
		</div>
	{:else if menu === Menu.Settings}
		<MenuButton on:click={() => (menu = Menu.Root)}>
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

		min-width: 16rem;
	}

	.profile {
		color: var(--text);
		text-decoration: none;
		padding: 1rem;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		display: grid;
		grid-template-columns: auto auto 1fr;
		grid-template-rows: auto auto;
		align-items: center;
		row-gap: 0.5rem;
		column-gap: 0.75rem;

		.profile-picture {
			grid-row: 1 / -1;

			width: 3rem;
			height: 3rem;
			border-radius: 50%;
			border: 2px solid var(--staff);
		}

		.name {
			grid-row: 1;
			font-size: 1rem;
			font-weight: 600;
			color: var(--staff);
		}

		.roles {
			grid-row: 2;

			display: flex;
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
