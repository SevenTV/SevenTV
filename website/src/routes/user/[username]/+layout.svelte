<script lang="ts">
	import Role from "$/components/profile/role.svelte";
	import Fa from "svelte-fa";
	import {
		faBolt,
		faBrush,
		faChevronDown,
		faCircleCheck,
		faComment,
		faEllipsisV,
		faFileLines,
		faFolder,
		faGear,
		faGift,
		faUpload,
	} from "@fortawesome/free-solid-svg-icons";
	import type { LayoutData } from "./$types";
	import { faHeart } from "@fortawesome/free-regular-svg-icons";
	import { faTwitch, faDiscord } from "@fortawesome/free-brands-svg-icons";
	import Expandable from "$/components/expandable.svelte";
	import { page } from "$app/stores";
	import HideOn from "$/components/hide-on.svelte";

	export let data: LayoutData;

    function scrollIntoView(e: MouseEvent) {
        if (e.target instanceof HTMLElement) {
            e.target.scrollIntoView({ behavior: "smooth", block: "center", inline: "center" });
        }
    }
</script>

<svelte:head>
	<title>{data.username} - 7TV</title>
</svelte:head>

<div class="layout">
	<div class="side-bar">
		<img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
        <span class="name">
            {data.username}
            <Fa icon={faCircleCheck} size="0.75x" />
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
			<button class="button secondary grow">
				<Fa icon={faHeart} size="1.2x" />
				Follow
			</button>
			<button class="button secondary more hide-on-mobile">
				<Fa icon={faChevronDown} size="1.2x" />
			</button>
			<button class="button secondary grow hide-on-desktop">
				<Fa icon={faGift} size="1.2x" />
                Gift
			</button>
		</div>
        <HideOn mobile>
            <Expandable title="Connections">
                <a class="button connection" target="_blank" href="https://twitch.tv/ayyybubu">
                    <Fa icon={faTwitch} size="1.2x" fw />
                    <span>ayyybubu</span>
                </a>
                <button class="button connection">
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
        <button class="button more hide-on-desktop">
            <Fa icon={faEllipsisV} size="1.2x" fw />
        </button>
	</div>
	<div class="content">
		<div class="header">
			<div class="tabs">
				<a
					href="/user/{data.username}"
					class="button no-bg"
                    draggable="false"
					class:secondary={$page.url.pathname === `/user/${data.username}`}
                    on:click={scrollIntoView}
				>
					<Fa icon={faBolt} size="1.2x" />
					Active
				</a>
				<a
					href="/user/{data.username}/uploaded"
					class="button no-bg"
                    draggable="false"
					class:secondary={$page.url.pathname === `/user/${data.username}/uploaded`}
                    on:click={scrollIntoView}
				>
					<Fa icon={faUpload} size="1.2x" />
					Uploaded
				</a>
				<a
					href="/user/{data.username}/emote-sets"
					class="button no-bg"
                    draggable="false"
					class:secondary={$page.url.pathname === `/user/${data.username}/emote-sets`}
                    on:click={scrollIntoView}
				>
					<Fa icon={faFolder} size="1.2x" />
					Emote Sets
				</a>
				<a
					href="/user/{data.username}/cosmetics"
					class="button no-bg"
                    draggable="false"
					class:secondary={$page.url.pathname === `/user/${data.username}/cosmetics`}
                    on:click={scrollIntoView}
				>
					<Fa icon={faBrush} size="1.2x" />
					Cosmetics
				</a>
				<a
					href="/user/{data.username}/activity-log"
					class="button no-bg"
                    draggable="false"
					class:secondary={$page.url.pathname === `/user/${data.username}/activity-log`}
                    on:click={scrollIntoView}
				>
					<Fa icon={faFileLines} size="1.2x" />
					Activity Log
				</a>
				<a
					href="/user/{data.username}/mod-comments"
					class="button no-bg"
                    draggable="false"
					class:secondary={$page.url.pathname === `/user/${data.username}/mod-comments`}
                    on:click={scrollIntoView}
				>
					<Fa icon={faComment} size="1.2x" />
					Mod Comments
				</a>
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
	.layout {
		display: flex;
		height: 100%;
	}

	.side-bar {
		min-width: 18rem;
		overflow: auto;
        position: relative;

		background-color: var(--bg-medium);
		padding: 1rem 1.25rem;

		display: flex;
		flex-direction: column;
		gap: 0.75rem;

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

		.connection {
			padding: 0.75rem;
			font-size: 0.875rem;
			font-weight: 500;

			gap: 0.75rem;
		}

        & > .more {
            position: absolute;
            top: 0.5rem;
            right: 1rem;
        }
	}

	.content {
		flex-grow: 1;
		overflow: auto;

		padding: 1rem 2rem;

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
        .layout {
            flex-direction: column;
            overflow: auto;
        }

        .side-bar {
            overflow: unset;
            margin: 0.75rem 1rem;
            border-radius: 0.5rem;
            padding: 1rem;

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
            overflow: unset;
            padding: 0.75rem 1rem;
            padding-top: 0;

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
