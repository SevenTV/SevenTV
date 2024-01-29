<script lang="ts">
    import Role from "$/components/profile/role.svelte";
    import Fa from "svelte-fa";
    import { faBolt, faBrush, faChevronDown, faCircleCheck, faComment, faFileLines, faFolder, faPaintBrush, faUpload } from "@fortawesome/free-solid-svg-icons";
    import type { LayoutData } from "./$types";
    import { faHeart } from "@fortawesome/free-regular-svg-icons";
    import { faTwitch, faDiscord } from "@fortawesome/free-brands-svg-icons";
    import Expandable from "$/components/expandable.svelte";
    import Tabs from "$/components/nav/tabs.svelte";
    import { page } from "$app/stores";

    export let data: LayoutData;
</script>

<svelte:head>
    <title>{data.username} - 7TV</title>
</svelte:head>

<div class="layout">
    <div class="side-bar">
        <img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
        <div class="profile">
            <span class="name">
                {data.username}
                <Fa icon={faCircleCheck} size="0.75x" />
            </span>
            <div class="roles">
                <Role name="Staff" />
                <Role name="Subscriber" />
            </div>
        </div>
        <div class="profile-data">
            <span>
                1.4k
                <br />
                <span class="text">followers</span>
            </span>
            <span>
                1.2M
                <br />
                <span class="text">channels</span>
            </span>
        </div>
        <div class="buttons">
            <button class="button secondary follow">
                <Fa icon={faHeart} size="1.2x" />
                Follow
            </button>
            <button class="button secondary more">
                <Fa icon={faChevronDown} size="1.2x" />
            </button>
        </div>
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
    </div>
    <div class="content">
        <div class="tabs">
            <a href="/user/{data.username}" class="button no-bg" class:secondary={$page.url.pathname === `/user/${data.username}`}>
                <Fa icon={faBolt} size="1.2x" />
                Active
            </a>
            <a href="/user/{data.username}/uploaded" class="button no-bg" class:secondary={$page.url.pathname === `/user/${data.username}/uploaded`}>
                <Fa icon={faUpload} size="1.2x" />
                Uploaded
            </a>
            <a href="/user/{data.username}/emote-sets" class="button no-bg" class:secondary={$page.url.pathname === `/user/${data.username}/emote-sets`}>
                <Fa icon={faFolder} size="1.2x" />
                Emote Sets
            </a>
            <a href="/user/{data.username}/cosmetics" class="button no-bg" class:secondary={$page.url.pathname === `/user/${data.username}/cosmetics`}>
                <Fa icon={faBrush} size="1.2x" />
                Cosmetics
            </a>
            <a href="/user/{data.username}/activity-log" class="button no-bg" class:secondary={$page.url.pathname === `/user/${data.username}/activity-log`}>
                <Fa icon={faFileLines} size="1.2x" />
                Activity Log
            </a>
            <a href="/user/{data.username}/mod-comments" class="button no-bg" class:secondary={$page.url.pathname === `/user/${data.username}/mod-comments`}>
                <Fa icon={faComment} size="1.2x" />
                Mod Comments
            </a>
        </div>
        <hr />
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

        .profile {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 0.5rem;

            .name {
                font-size: 1.125rem;
                font-weight: 600;
                color: var(--staff);
            }

            .roles {
                display: flex;
                gap: 0.25rem;
            }
        }

        .profile-data {
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

            & > .follow {
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
    }

    .content {
        flex-grow: 1;
        overflow: auto;

        padding: 1rem 2rem;

        .tabs {
            display: flex;
            gap: 0.5rem;
        }

        hr {
            margin: 1rem 0;
        }
    }
</style>
