<script lang="ts">
	import TabLink from "../tab-link.svelte";
	import { Users, ChartLineUp, Graph, ChatText, Note, Pulse } from "phosphor-svelte";

	export let id: string;
</script>

<nav class="links">
	<TabLink title="Channels (1020)" href="/emotes/{id}" responsive>
		<Users />
		<Users weight="fill" slot="active" />
	</TabLink>
	<TabLink title="Activity" href="/emotes/{id}/activity" responsive>
		<Pulse />
		<Pulse weight="fill" slot="active" />
	</TabLink>
	<TabLink title="Statistics" href="/emotes/{id}/statistics" responsive>
		<ChartLineUp />
		<ChartLineUp weight="fill" slot="active" />
	</TabLink>
	<TabLink title="Suggested Emotes" href="/emotes/{id}/suggested-emotes" responsive>
		<Graph />
		<Graph weight="fill" slot="active" />
	</TabLink>
	<TabLink title="Mod Comments" href="/emotes/{id}/mod-comments" responsive>
		<ChatText />
		<ChatText weight="fill" slot="active" />
	</TabLink>
</nav>

<style lang="scss">
	.links {
		display: flex;
		border-radius: 0.5rem;
		background-color: var(--bg-light);

		-ms-overflow-style: none;
		scrollbar-width: none;
		&::-webkit-scrollbar {
			display: none;
		}
	}
</style>
