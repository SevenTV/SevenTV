<script lang="ts">
	import { Theme, theme, user } from "$/lib/stores";
	import Role from "../profile/role.svelte";
	import { fade } from "svelte/transition";
	import {
		CaretRight,
		ChartLine,
		ChatDots,
		Code,
		Compass,
		Gear,
		GlobeHemisphereWest,
		House,
		LockSimple,
		Moon,
		Note,
		PaintBrush,
		Question,
		SealCheck,
		SignOut,
		Sliders,
		Smiley,
		Star,
		Sun,
	} from "phosphor-svelte";
	import MenuButton from "../input/menu-button.svelte";

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
				Home
			</MenuButton>
			<MenuButton href="/emotes">
				<Smiley />
				Emotes
			</MenuButton>
			<MenuButton href="/discover">
				<Compass />
				Discover
			</MenuButton>
			<MenuButton href="/store" style="color: var(--store)">
				<Star />
				Store
			</MenuButton>
		</div>
		{#if $user}
			<div class="link-list">
				<MenuButton href="/cosmetics">
					<PaintBrush />
					Cosmetics
				</MenuButton>
				<MenuButton href="/analytics">
					<ChartLine />
					Analytics
				</MenuButton>
			</div>
			<hr class="hide-on-mobile" />
		{/if}
		<div class="link-list">
			<MenuButton showCaret on:click={(e) => setMenu(e, Menu.Language)}>
				<GlobeHemisphereWest />
				Language
			</MenuButton>
			<MenuButton showCaret on:click={(e) => setMenu(e, Menu.Theme)}>
				<Moon />
				Theme
			</MenuButton>
			{#if $user}
				<MenuButton href="/settings" hideOnMobile>
					<Gear />
					Settings
				</MenuButton>
				<MenuButton showCaret hideOnDesktop on:click={(e) => setMenu(e, Menu.Settings)}>
					<Gear />
					Settings
				</MenuButton>
			{/if}
		</div>
		<hr class="hide-on-mobile" />
		<div class="link-list">
			<MenuButton href="https://7tv.io/">
				<Code />
				Developer Portal
			</MenuButton>
			<MenuButton href="/contact">
				<ChatDots />
				Contact
			</MenuButton>
			<MenuButton href="/faq">
				<Question />
				FAQ
			</MenuButton>
			<MenuButton href="/privacy">
				<LockSimple />
				Privacy Policy
			</MenuButton>
			<MenuButton href="/tos">
				<Note />
				Terms of Service
			</MenuButton>
		</div>
		{#if $user}
			<hr class="hide-on-mobile" />
			<div class="link-list">
				<MenuButton on:click={() => ($user = false)}>
					<SignOut />
					Sign out
				</MenuButton>
			</div>
		{/if}
	{:else if menu === Menu.Language}
		Language Picker
	{:else if menu === Menu.Theme}
		<h2>Theme</h2>
		<div class="link-list">
			<MenuButton on:click={() => ($theme = Theme.System)}>
				<Sliders />
				System
			</MenuButton>
			<MenuButton on:click={() => ($theme = Theme.Dark)}>
				<Moon />
				Dark
			</MenuButton>
			<MenuButton on:click={() => ($theme = Theme.Light)}>
				<Sun />
				Light
			</MenuButton>
		</div>
	{:else if menu === Menu.Settings}
		Settings
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

	h2 {
		margin: 0.5rem 1.2rem;
		font-size: 1.25rem;
		font-weight: 600;
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
