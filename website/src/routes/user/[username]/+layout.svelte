<script lang="ts">
	import Role from "$/components/profile/role.svelte";
	import type { LayoutData } from "./$types";
	import TabLink from "$/components/tab-link.svelte";
	import {
		CaretDown,
		CaretRight,
		ChartLineUp,
		ChatCircleText,
		DiscordLogo,
		DotsThreeVertical,
		FolderSimple,
		Gift,
		Heart,
		IdentificationCard,
		Lightning,
		Link,
		PaintBrush,
		Pulse,
		SealCheck,
		TwitchLogo,
		TwitterLogo,
		Upload,
		UserCircle,
		YoutubeLogo,
	} from "phosphor-svelte";
	import Button from "$/components/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";

	export let data: LayoutData;

	let connectionsExpanded = false;
	let editorsExpanded = false;
</script>

<svelte:head>
	<title>{data.username} - 7TV</title>
</svelte:head>

<div class="side-bar-layout">
	<aside class="side-bar">
		<img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
		<span class="name">
			{data.username}
			<SealCheck size="0.8rem" />
		</span>
		<div class="roles">
			<Role name="Staff" />
			<Role name="Subscriber" />
		</div>
		<div class="data">
			<span>
				1.4k
				<br class="hide-on-mobile" />
				<span class="text">followers</span>
			</span>
			<span>
				1.2M
				<br class="hide-on-mobile" />
				<span class="text">channels</span>
			</span>
		</div>
		<div class="buttons">
			<Button primary style="flex-grow: 1; justify-content: center;">
				<Heart slot="icon" />
				Follow
			</Button>
			<Button secondary hideOnMobile>
				<CaretDown slot="icon" />
			</Button>
			<Button secondary hideOnDesktop>
				<Gift slot="icon" />
				Gift
			</Button>
		</div>
		<nav class="link-list hide-on-mobile">
			<Button big on:click={() => (connectionsExpanded = !connectionsExpanded)}>
				<Link slot="icon" />
				Connections
				{#if connectionsExpanded}
					<CaretDown slot="icon-right" style="margin-left: auto" />
				{:else}
					<CaretRight slot="icon-right" style="margin-left: auto" />
				{/if}
			</Button>
			{#if connectionsExpanded}
				<div class="expanded">
					<Button href="https://twitch.tv/ayyybubu" target="_blank">
						<TwitchLogo slot="icon" />
						<span>ayyybubu</span>
					</Button>
					<Button href="https://youtube.com/channel/bubutv" target="_blank">
						<YoutubeLogo slot="icon" />
						<span>bubutv</span>
					</Button>
					<Button href="https://twitter.com/tweetbubu" target="_blank">
						<TwitterLogo slot="icon" />
						<span>tweetbubu</span>
					</Button>
				</div>
			{/if}
			<Button big on:click={() => (editorsExpanded = !editorsExpanded)}>
				<UserCircle slot="icon" />
				Editors
				{#if editorsExpanded}
					<CaretDown slot="icon-right" style="margin-left: auto"/>
				{:else}
					<CaretRight slot="icon-right" style="margin-left: auto" />
				{/if}
			</Button>
			{#if editorsExpanded}
				<div class="expanded">
					<ChannelPreview size={1.5} />
					<ChannelPreview size={1.5} />
				</div>
			{/if}
			<hr />
			<TabLink title="Active Emotes" href="/user/{data.username}" big>
				<Lightning />
				<Lightning weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Uploaded Emotes" href="/user/{data.username}/uploaded" big>
				<Upload />
				<Upload weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Emote Sets" href="/user/{data.username}/emote-sets" big>
				<FolderSimple />
				<FolderSimple weight="fill" slot="active" />
			</TabLink>
			<hr />
			<TabLink title="Cosmetics" href="/user/{data.username}/cosmetics" big>
				<PaintBrush />
				<PaintBrush weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Activity" href="/user/{data.username}/activity" big>
				<Pulse />
				<Pulse weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Analytics" href="/user/{data.username}/analytics" big>
				<ChartLineUp />
				<ChartLineUp weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Mod Comments" href="/user/{data.username}/mod-comments" big>
				<ChatCircleText />
				<ChatCircleText weight="fill" slot="active" />
			</TabLink>
		</nav>
		<Button hideOnDesktop style="position: absolute; top: 0.5rem; right: 1rem;">
			<DotsThreeVertical slot="icon" />
		</Button>
	</aside>
	<div class="content">
		<div class="header hide-on-desktop">
			<nav class="tabs">
				<TabLink title="About" href="/user/{data.username}/about">
					<IdentificationCard />
					<IdentificationCard weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Active" href="/user/{data.username}">
					<Lightning />
					<Lightning weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Uploaded" href="/user/{data.username}/uploaded">
					<Upload />
					<Upload weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Emote Sets" href="/user/{data.username}/emote-sets">
					<FolderSimple />
					<FolderSimple weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Cosmetics" href="/user/{data.username}/cosmetics">
					<PaintBrush />
					<PaintBrush weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Activity" href="/user/{data.username}/activity">
					<Pulse />
					<Pulse weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Analytics" href="/user/{data.username}/analytics">
					<ChartLineUp />
					<ChartLineUp weight="fill" slot="active" />
				</TabLink>
				<TabLink title="Mod Comments" href="/user/{data.username}/mod-comments">
					<ChatCircleText />
					<ChatCircleText weight="fill" slot="active" />
				</TabLink>
			</nav>
		</div>
		<div class="page">
			<slot />
		</div>
	</div>
</div>

<style lang="scss">
	.side-bar {
		.profile-picture {
			align-self: center;

			width: 4.75rem;
			height: 4.75rem;
			border-radius: 50%;
			border: 2px solid var(--staff);
		}

		.name {
			align-self: center;

			font-size: 1.125rem;
			font-weight: 600;
			color: var(--staff);
		}

		.roles {
			align-self: center;

			display: flex;
			gap: 0.25rem;
		}

		.data {
			align-self: center;

			display: flex;
			gap: 2rem;

			font-size: 0.875rem;
			font-weight: 600;
			text-align: center;

			.text {
				font-weight: 400;
				color: var(--text-light);
			}
		}

		.buttons {
			align-self: stretch;

			display: flex;
			gap: 0.5rem;
			flex-wrap: wrap;
		}

		// Select all buttons except the active one
		.link-list > :global(.button:not(.secondary)) {
			color: var(--text-light);
			font-weight: 500;
		}

		.expanded {
			margin-left: 0.5rem;

			display: flex;
			flex-direction: column;
			gap: 0.5rem;
		}
	}

	.content {
		display: flex;
		flex-direction: column;

		.header {
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 0.5rem;
		}

		.tabs {
			display: flex;
			border-radius: 0.5rem;
			background-color: var(--bg-light);
			overflow: auto;

			-ms-overflow-style: none;
			scrollbar-width: none;
			&::-webkit-scrollbar {
				display: none;
			}
		}

		.page {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;
		}
	}

	@media screen and (max-width: 960px) {
		.side-bar {
			display: grid;
			grid-template-columns: auto 1fr;
			grid-template-rows: auto auto auto auto;
			row-gap: 0.5rem;
			column-gap: 1rem;

			.profile-picture {
				grid-row: 1 / span 3;
				grid-column: 1;
			}

			.name {
				grid-row: 1;
				grid-column: 2;
			}

			.roles {
				grid-row: 2;
				grid-column: 2;
			}

			.data {
				grid-row: 3;
				grid-column: 2;

				gap: 1rem;
			}

			.buttons {
				grid-row: 4;
				grid-column: 1 / span 2;

				margin-top: 0.5rem;
			}
		}

		.content .header {
			margin-bottom: 0.75rem;
		}
	}
</style>
