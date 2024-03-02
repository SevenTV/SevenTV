<script lang="ts">
	import Button from "$/components/button.svelte";
    import EmoteContainer from "$/components/emote-container.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
	import Tags from "$/components/emotes/tags.svelte";
	import { Copy, Lightning, LightningSlash, NotePencil, Trash } from "phosphor-svelte";
    import type { PageData } from "./$types";
	import Select from "$/components/select.svelte";
	import SearchBar from "$/components/search-bar.svelte";
	import { Layout, emotesLayout } from "$/lib/stores";
	import LayoutButtons from "$/components/emotes/layout-buttons.svelte";
	import Toggle from "$/components/toggle.svelte";
	import Flags from "$/components/emotes/flags.svelte";
    import HideOn from "$/components/hide-on.svelte";

    export let data: PageData;

    let enabled = false;
    let selectionMode = false;
</script>

<svelte:head>
    <title>{data.id} - 7TV</title>
</svelte:head>

<div class="layout">
    <div class="set-info">
        <h1>{data.id}</h1>
        <Flags flags={["verified", "public"]} style="position: absolute; top: 1rem; right: 1rem;" />
        <Tags tags={["lorem", "tag"]} />
        <div class="progress">
            <progress max="600" value="100" />
            100/600
        </div>
    </div>
    <div class="controls">
        <div class="buttons">
            <Button secondary on:click={() => (selectionMode = !selectionMode)} hideOnDesktop>
                Select
                <Toggle bind:value={selectionMode} numb slot="icon-right" />
            </Button>
            <HideOn mobile={selectionMode}>
                <Button primary on:click={() => (enabled = !enabled)}>
                    {#if enabled}
                        Disable
                    {:else}
                        Enable
                    {/if}
                    <svelte:fragment slot="icon-right">
                        {#if enabled}
                            <LightningSlash />
                        {:else}
                            <Lightning />
                        {/if}
                    </svelte:fragment>
                </Button>
            </HideOn>
            <Button secondary hideOnMobile>
                Edit
                <NotePencil slot="icon-right" />
            </Button>
            <Button secondary hideOnMobile>
                Copy Set
                <Copy slot="icon-right" />
            </Button>
            {#if !selectionMode}
                <Button secondary hideOnDesktop>
                    <NotePencil slot="icon-right" />
                </Button>
                <Button secondary hideOnDesktop>
                    <Copy slot="icon-right" />
                </Button>
            {/if}
            <Button secondary on:click={() => (selectionMode = !selectionMode)} hideOnMobile>
                Selection Mode
                <Toggle bind:value={selectionMode} numb slot="icon-right" />
            </Button>
            {#if selectionMode}
                <Button>
                    <Copy slot="icon" />
                </Button>
                <Button>
                    <NotePencil slot="icon" />
                </Button>
                <Button>
                    <Trash slot="icon" />
                </Button>
            {/if}
        </div>
        <div class="buttons">
            <Select options={["No Filters", "Filters"]} />
            <SearchBar grow />
            <LayoutButtons />
        </div>
    </div>
    <div class="content">
        <EmoteContainer layout={$emotesLayout}>
            {#each Array(100) as _, i}
                <EmotePreview name="emoteSetEmote{i}" index={i} emoteOnly={$emotesLayout === Layout.SmallGrid} />
            {/each}
        </EmoteContainer>
    </div>
</div>

<style lang="scss">
    .layout {
        padding: 1.25rem;
        padding-bottom: 0;
        height: 100%;

        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    progress[value] {
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
        border: none;

        width: 100%;
        height: 0.5rem;

        &, &::-webkit-progress-bar {
            border-radius: 0.25rem;
            background-color: var(--secondary);
        }

        &::-moz-progress-bar {
            border-radius: 0.25rem;
            background-color: var(--primary);
        }

        &::-webkit-progress-value {
            border-radius: 0.25rem;
            background-color: var(--primary);
        }
    }

    .set-info {
        position: relative;
        padding: 1rem;

        display: flex;
        flex-direction: column;
        gap: 0.75rem;

        background-color: var(--bg-medium);
        border-radius: 0.5rem;

        h1 {
            text-align: center;
            font-size: 1.125rem;
            font-weight: 500;
        }

        .progress {
            display: flex;
            align-items: center;
            gap: 0.75rem;

            font-size: 0.875rem;
            font-weight: 500;

            progress {
                flex-grow: 1;
            }
        }
    }

    .controls {
        display: flex;
        gap: 0.5rem;
        flex-wrap: wrap-reverse;
        justify-content: space-between;
    }

    .buttons {
        display: flex;
        gap: 0.5rem;
        align-items: center;
    }

    .content {
        overflow: auto;
        overflow: overlay;
        scrollbar-gutter: stable;
        margin-right: -1.25rem;
        padding-right: 1.25rem;
    }

    @media screen and (max-width: 960px) {
        .layout {
            padding: 0.5rem;
            // Scroll whole layout on mobile
            height: auto;
        }

        .content {
            margin-right: 0;
            padding-right: 0;
        }
    }
</style>
