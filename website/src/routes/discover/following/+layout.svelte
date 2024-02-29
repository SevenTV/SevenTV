<script lang="ts">
	import Button from "$/components/button.svelte";
    import TabLink from "$/components/tab-link.svelte";
	import { Layout, discoverFollowingLayout } from "$/lib/stores";
	import { ListDashes, GridFour, Pulse, Upload } from "phosphor-svelte";
</script>

<svelte:head>
	<title>Following - 7TV</title>
</svelte:head>

<div class="nav">
    <div class="tabs">
        <TabLink title="Uploads" href="/discover/following">
            <Upload />
            <Upload weight="fill" slot="active" />
        </TabLink>
        <TabLink title="Activity" href="/discover/following/activity">
            <Pulse />
            <Pulse weight="fill" slot="active" />
        </TabLink>
    </div>
    <div class="buttons">
        <Button secondary={$discoverFollowingLayout === Layout.List} on:click={() => ($discoverFollowingLayout = Layout.List)}>
            <ListDashes slot="icon" />
        </Button>
        <Button secondary={$discoverFollowingLayout === Layout.BigGrid} on:click={() => ($discoverFollowingLayout = Layout.BigGrid)}>
            <GridFour slot="icon" />
        </Button>
    </div>
</div>
<slot />

<style lang="scss">
    .nav {
        display: flex;
        justify-content: space-between;
    }

    .tabs {
		display: flex;
		align-items: center;
		background-color: var(--bg-light);
		border-radius: 0.5rem;
	}

    .buttons {
        display: flex;
        gap: 0.5rem;
    }
</style>
