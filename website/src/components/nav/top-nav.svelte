<script lang="ts">
	import Logo from "$/components/icons/logo.svelte";
	import SearchBar from "$/components/search-bar.svelte";
	import TopTabs from "./top-tabs.svelte";
	import Badge from "../badge.svelte";
	import HideOn from "../hide-on.svelte";
	import { user, showMobileMenu, showUploadDialog } from "$/lib/stores";
	import DropDown from "../drop-down.svelte";
	import Menu from "./menu.svelte";
	import Dms from "../dms.svelte";
	import Notifications from "../notifications.svelte";
	import { Bell, Chat, List, MagnifyingGlass, Plus, PlusSquare } from "phosphor-svelte";
	import Button from "../button.svelte";
</script>

<nav>
	<div class="links">
		<a class="home" href="/">
			<Logo />
		</a>
		<HideOn mobile>
			<TopTabs
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
		<Button hideOnDesktop>
			<MagnifyingGlass slot="icon" />
		</Button>
		{#if $user}
			<DropDown>
				<Button>
					<Bell slot="icon" />
				</Button>
				<Notifications slot="dropdown" />
			</DropDown>
			<DropDown>
				<Button>
					<Badge count={1} slot="icon">
						<Chat />
					</Badge>
				</Button>
				<Dms slot="dropdown" />
			</DropDown>
			<Button hideOnDesktop on:click={() => ($showUploadDialog = true)}>
				<PlusSquare slot="icon" />
			</Button>
			<Button primary hideOnMobile on:click={() => ($showUploadDialog = true)}>
				<PlusSquare slot="icon" />
				Upload
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
			<Button href="/sign-in" secondary>Sign In</Button>
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
