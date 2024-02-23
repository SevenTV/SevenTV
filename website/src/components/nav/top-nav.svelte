<script lang="ts">
	import Logo from "$/components/icons/logo.svelte";
	import SearchBar from "$/components/nav/search-bar.svelte";
	import Tabs from "./tabs.svelte";
	import Fa from "svelte-fa";
	import { faBell, faMessage, faPlusSquare } from "@fortawesome/pro-regular-svg-icons";
	import { faBars, faPlus, faSearch } from "@fortawesome/pro-solid-svg-icons";
	import Badge from "../badge.svelte";
	import HideOn from "../hide-on.svelte";
	import { user, showMobileMenu } from "$/lib/stores";
	import DropDown from "../drop-down.svelte";
	import Menu from "./menu.svelte";
	import Dms from "../dms.svelte";
	import Notifications from "../notifications.svelte";
	import { Bell, Chat, List, MagnifyingGlass, Plus, PlusSquare } from "phosphor-svelte";
</script>

<nav>
	<div class="links">
		<a class="home" href="/">
			<Logo />
		</a>
		<HideOn mobile>
			<Tabs
				tabs={[
					{ name: "Emotes", pathname: "/emotes" },
					{ name: "Discover", pathname: "/discover" },
					{ name: "Store", pathname: "/store", hightlight: true },
				]}
			/>
		</HideOn>
	</div>
	<HideOn mobile>
		<SearchBar big />
	</HideOn>
	<div class="user-actions">
		<button class="button hide-on-desktop square">
			<MagnifyingGlass />
		</button>
		{#if $user}
			<DropDown button>
				<Bell />
				<Notifications slot="dropdown" />
			</DropDown>
			<DropDown button>
				<Badge count={1}>
					<Chat />
				</Badge>
				<Dms slot="dropdown" />
			</DropDown>
			<a href="/upload" class="button square hide-on-desktop">
				<PlusSquare />
			</a>
			<a href="/upload" class="button icon-left secondary hide-on-mobile">
				<Plus />
				Upload
			</a>
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
				<DropDown button>
					<List />
					<Menu slot="dropdown" />
				</DropDown>
			</HideOn>
			<a class="button primary" href="/sign-in">Sign In</a>
		{/if}
		<!-- Only show when logged out on mobile -->
		{#if !$user}
			<button class="button square hide-on-desktop" on:click={() => ($showMobileMenu = !$showMobileMenu)}>
				<List />
			</button>
		{/if}
	</div>
</nav>

<style lang="scss">
	nav {
		background-color: var(--bg-dark);
		border-bottom: 1px solid var(--border);
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
