<script lang="ts">
	import TabLink from "../tab-link.svelte";
    import {
		Users,
		ChartLineUp,
		Graph,
		ChatText,
		Pulse,
	} from "phosphor-svelte";

    export let id: string;
</script>

<div class="links">
    <TabLink title="Channels (1020)" href="/emotes/{id}" first>
        <Users />
        <Users weight="fill" slot="active" />
    </TabLink>
    <TabLink title="Activity" href="/emotes/{id}/activity">
        <Pulse />
        <Pulse weight="fill" slot="active" />
    </TabLink>
    <TabLink title="Statistics" href="/emotes/{id}/statistics">
        <ChartLineUp />
        <ChartLineUp weight="fill" slot="active" />
    </TabLink>
    <TabLink title="Suggested Emotes" href="/emotes/{id}/suggested-emotes">
        <Graph />
        <Graph weight="fill" slot="active" />
    </TabLink>
    <TabLink title="Mod Comments" href="/emotes/{id}/mod-comments" last>
        <ChatText />
        <ChatText weight="fill" slot="active" />
    </TabLink>
</div>

<style lang="scss">
    .links {
        display: flex;
        border-radius: 0.5rem;
        background-color: var(--bg-light);
    }
</style>
