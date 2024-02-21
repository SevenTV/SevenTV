<script lang="ts">
	import Tabs from "$/components/tabs.svelte";
	import { sideBar } from "$/lib/stores";
	import { faArrowRightToLine, faFolder } from "@fortawesome/pro-regular-svg-icons";
	import { faGrid, faGridHorizontal } from "@fortawesome/pro-solid-svg-icons";
	import Fa from "svelte-fa";
</script>

<div class="nav-bar">
    <div class="buttons">
        {#if !$sideBar}
            <button class="button square expand" on:click={() => ($sideBar = true)}>
                <Fa icon={faArrowRightToLine} size="1.2x" fw />
            </button>
        {/if}
        <Tabs tabs={[
            { name: "Top", pathname: "/emotes/top" },
            { name: "Trending", pathname: "/emotes" },
        ]} />
    </div>
    <div class="buttons">
        <button class="button secondary icon-left hide-on-mobile">
            <Fa icon={faFolder} fw size="1.2x" />
            Personal Emotes
        </button>
        <button class="button square">
            <Fa icon={faGridHorizontal} fw />
        </button>
        <button class="button square">
            <Fa icon={faGrid} fw />
        </button>
    </div>
</div>
<slot />

<style lang="scss">
	.nav-bar {
		display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
	}

    .buttons {
        display: flex;
        gap: 0.5rem;
    }
</style>
