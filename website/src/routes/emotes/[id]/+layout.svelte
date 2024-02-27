<script lang="ts">
	import Button from "$/components/button.svelte";
	import Activity from "$/components/emotes/activity.svelte";
	import Tag from "$/components/emotes/tag.svelte";
	import ImagePreview from "$/components/image-preview.svelte";
	import TabLink from "$/components/profile/tab-link.svelte";
    import {
		ArrowBendDownRight,
		Plus,
		FolderPlus,
		NotePencil,
		CaretDown,
		Users,
		ChartLineUp,
		Graph,
		ChatText,
		GlobeSimple,
		Flame,
		StackSimple,
	} from "phosphor-svelte";
</script>

<svelte:head>
    <title>emoteName - 7TV</title>
</svelte:head>

<div class="grid">
    <div>
        <div class="top-bar">
            <a href="/user/ayyybubu" class="user-info">
                <img src="/test-profile-pic.jpeg" alt="profile" class="profile-picture" />
                <span class="username">ayyybubu</span>
                <div class="artists">
                    <ArrowBendDownRight size="0.75rem" color="var(--primary-light)" />
                    <a href="/user/ayyybubu" class="profile">
                        <img
                            src="/test-profile-pic.jpeg"
                            alt="ayyybubu"
                            title="ayyybubu"
                            class="artist-picture"
                        />
                    </a>
                </div>
            </a>
            <div class="flags">
                <span class="flag">
                    <GlobeSimple size="1rem" />
                    Global
                </span>
                <span class="flag">
                    <Flame size="1rem" />
                    Trending
                </span>
                <span class="flag">
                    <StackSimple size="1rem" />
                    Overlay
                </span>
            </div>
        </div>

        <div class="emote-info">
            <div class="heading">
                <h1>emoteName</h1>
                <div class="tags">
                    <Tag>lorem</Tag>
                    <Tag>ipsum</Tag>
                    <Tag>dolor</Tag>
                </div>
            </div>
            <div class="previews">
                <ImagePreview size={32} />
                <ImagePreview size={64} />
                <ImagePreview size={96} />
                <ImagePreview size={128} />
            </div>
            <div class="buttons">
                <Button secondary>
                    <Plus slot="icon" />
                    <span>
                        Use
                        <span class="hide-on-mobile">Emote</span>
                    </span>
                </Button>
                <Button primary>
                    <FolderPlus slot="icon" />
                    Add to...
                </Button>
                <Button primary>
                    <NotePencil slot="icon" />
                    Edit
                </Button>
                <Button primary hideOnMobile>
                    More
                    <CaretDown slot="icon-right" />
                </Button>
                <Button primary hideOnDesktop>
                    <CaretDown slot="icon" />
                </Button>
            </div>
        </div>
    </div>
    <div class="activity">
        <h2>Activity</h2>
        <div class="activity-events">
            <Activity activities={[]} />
        </div>
    </div>
    <div class="tabs">
        <div class="links">
            <TabLink title="Channels (1020)" href="/emotes/id">
                <Users />
                <svelte:fragment slot="active">
                    <Users weight="fill" />
                </svelte:fragment>
            </TabLink>
            <TabLink title="Statistics" href="/emotes/id/statistics">
                <ChartLineUp />
                <svelte:fragment slot="active">
                    <ChartLineUp weight="fill" />
                </svelte:fragment>
            </TabLink>
            <TabLink title="Suggested Emotes" href="/emotes/id/suggested-emotes">
                <Graph />
                <svelte:fragment slot="active">
                    <Graph weight="fill" />
                </svelte:fragment>
            </TabLink>
            <TabLink title="Mod Comments" href="/emotes/id/mod-comments">
                <ChatText />
                <svelte:fragment slot="active">
                    <ChatText weight="fill" />
                </svelte:fragment>
            </TabLink>
        </div>
        <slot />
    </div>
</div>

<style lang="scss">
    .grid {
        padding: 1.25rem;
        min-height: 100%;

        display: grid;
        grid-template-columns: 3fr 1fr;
        grid-template-rows: auto 1fr;
        gap: 1rem;

        & > * {
            background-color: var(--bg-medium);
            border-radius: 0.5rem;
            padding: 1rem;
        }
    }

    .top-bar {
        align-self: stretch;

        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
    }

    .user-info {
        display: grid;
        grid-template-columns: auto auto;
        grid-template-rows: auto auto;
        align-items: center;
        column-gap: 0.5rem;
        row-gap: 0.25rem;
        text-decoration: none;

        .profile-picture {
            grid-row: 1 / -1;

            width: 2.75rem;
            height: 2.75rem;
            border-radius: 50%;
            border: 2px solid var(--staff);
        }

        .username {
            color: var(--staff);
            font-weight: 500;
        }

        .artists {
            display: flex;
            gap: 0.25rem;

            .artist-picture {
                width: 1rem;
                height: 1rem;
                border-radius: 50%;
                border: 1px solid var(--text);
            }
        }
    }

    .flags {
        display: flex;
        align-items: center;
        gap: 0.5rem;

        .flag {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.3rem 0.5rem 0.3rem 0.75rem;
            background-color: var(--primary);
            border-radius: 0.5rem;

            color: var(--text-lighter);
            font-size: 0.75rem;
            font-weight: 500;
        }
    }

    .emote-info {
        margin-top: 0.5rem;

        .heading {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 1rem;

            h1 {
                font-size: 1.25rem;
                font-weight: 600;
            }

            .tags {
                display: flex;
                gap: 0.5rem;
            }
        }

        display: flex;
        flex-direction: column;
        justify-content: space-between;
        align-items: center;
        gap: 2rem;

        .previews {
            display: flex;
            align-items: flex-end;
            gap: 2.25rem;
        }

        .buttons {
            display: flex;
            gap: 0.5rem;
        }
    }

    .activity {
        min-width: 22rem;

        h2 {
            font-size: 1rem;
            font-weight: 600;
        }
    }

    .tabs {
        grid-column: 1 / -1;
        position: relative;

        .links {
            display: flex;
            gap: 0.5rem;
            flex-wrap: wrap;
        }
    }

    @media screen and (max-width: 960px) {
        .grid {
            grid-template-columns: 1fr;
            grid-template-rows: repeat(3, auto);
        }
    }
</style>
