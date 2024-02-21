<script lang="ts">
	import Role from "$/components/profile/role.svelte";
	import Fa from "svelte-fa";
	import {
		faBolt,
		faBrush,
		faChevronDown,
		faBadgeCheck,
		faComment,
		faEllipsisV,
		faMemo,
		faFolder,
		faUpload,
	} from "@fortawesome/pro-solid-svg-icons";
	import {
		faBolt as faRegularBolt,
		faBrush as faRegularBrush,
		faComment as faRegularComment,
		faMemo as faRegularMemo,
		faFolder as faRegularFolder,
		faGear,
		faUpload as faRegularUpload,
	} from "@fortawesome/pro-regular-svg-icons";
	import type { LayoutData } from "./$types";
	import { faGift, faHeart } from "@fortawesome/pro-regular-svg-icons";
	import { faTwitch, faDiscord } from "@fortawesome/free-brands-svg-icons";
	import Expandable from "$/components/expandable.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import TabLink from "$/components/profile/tab-link.svelte";

	export let data: LayoutData;
</script>

<svelte:head>
	<title>{data.username} - 7TV</title>
</svelte:head>

<div class="side-bar-layout">
	<aside class="side-bar">
		<img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
		<span class="name">
			{data.username}
			<Fa icon={faBadgeCheck} size="0.75x" />
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
			<button class="button icon-left secondary grow">
				<Fa icon={faHeart} size="1.2x" />
				Follow
			</button>
			<button class="button secondary square more hide-on-mobile">
				<Fa icon={faChevronDown} size="1.2x" fw />
			</button>
			<button class="button icon-left secondary grow hide-on-desktop">
				<Fa icon={faGift} size="1.2x" />
				Gift
			</button>
		</div>
		<HideOn mobile>
			<Expandable title="Connections">
				<a class="button big" target="_blank" href="https://twitch.tv/ayyybubu">
					<Fa icon={faTwitch} size="1.2x" fw />
					<span>ayyybubu</span>
				</a>
				<button class="button big">
					<Fa icon={faDiscord} size="1.2x" fw />
					<span>bubu</span>
				</button>
			</Expandable>
			<Expandable title="Editors">
				username
				<br />
				username
			</Expandable>
		</HideOn>
		<button class="button square more hide-on-desktop">
			<Fa icon={faEllipsisV} size="1.2x" fw />
		</button>
	</aside>
	<div class="content">
		<div class="header">
			<div class="tabs">
				<TabLink
					title="Active"
					href="/user/{data.username}"
					icon={faRegularBolt}
					activeIcon={faBolt}
				/>
				<TabLink
					title="Uploaded"
					href="/user/{data.username}/uploaded"
					icon={faRegularUpload}
					activeIcon={faUpload}
				/>
				<TabLink
					title="Emote Sets"
					href="/user/{data.username}/emote-sets"
					icon={faRegularFolder}
					activeIcon={faFolder}
				/>
				<TabLink
					title="Cosmetics"
					href="/user/{data.username}/cosmetics"
					icon={faRegularBrush}
					activeIcon={faBrush}
				/>
				<TabLink
					title="Activity Log"
					href="/user/{data.username}/activity-log"
					icon={faRegularMemo}
					activeIcon={faMemo}
				/>
				<TabLink
					title="Mod Comments"
					href="/user/{data.username}/mod-comments"
					icon={faRegularComment}
					activeIcon={faComment}
				/>
			</div>
			<a href="/settings" class="button no-bg hide-on-mobile">
				<Fa icon={faGear} size="1.2x" />
				Settings
			</a>
		</div>
		<hr class="hide-on-mobile" />
		<slot />
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
				color: var(--text-lighter);
			}
		}

		.buttons {
			align-self: stretch;

			display: flex;
			gap: 0.5rem;
			flex-wrap: wrap;

			& > .grow {
				flex-basis: 0;
				flex-grow: 1;
				justify-content: center;
			}

			& > .more {
				padding: 0.5rem;
			}
		}

		& > .more {
			position: absolute;
			top: 0.5rem;
			right: 1rem;
		}
	}

	.content {
		.header {
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 0.5rem;

			// All buttons except the selected one
			.button:not(.secondary) {
				color: var(--text-lighter);
			}
		}

		.tabs {
			display: flex;
			flex-wrap: wrap;
			gap: 0.5rem;
			user-select: none;

			-ms-overflow-style: none;
			scrollbar-width: none;
			&::-webkit-scrollbar {
				display: none;
			}
		}

		hr {
			margin: 1rem 0;
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

		.content {
			.header {
				margin-bottom: 0.75rem;
			}

			.tabs {
				gap: 0;
				margin-right: -1rem;
				padding-right: 1rem;
				overflow-x: auto;
				flex-wrap: nowrap;
			}
		}
	}
</style>
