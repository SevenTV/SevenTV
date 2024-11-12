<script lang="ts">
	import { numberFormat } from "$/lib/utils";
	import TabLink from "../tab-link.svelte";
	import { Users, Pulse } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	let { id, channelCount }: { id: string; channelCount?: number } = $props();

	let channelTabTitle = $derived(
		$t("common.channels", { values: { count: channelCount ?? 2 } }) +
			(channelCount ? ` (${numberFormat().format(channelCount)})` : ""),
	);
</script>

<nav class="links">
	<TabLink title={channelTabTitle} href="/emotes/{id}" responsive>
		<Users />
		{#snippet active()}
			<Users weight="fill" />
		{/snippet}
	</TabLink>
	<TabLink title={$t("common.activity")} href="/emotes/{id}/activity" responsive>
		<Pulse />
		{#snippet active()}
			<Pulse weight="fill" />
		{/snippet}
	</TabLink>
	<!-- <TabLink title={$t("common.statistics")} href="/emotes/{id}/statistics" responsive>
		<ChartLineUp />
		<ChartLineUp weight="fill" />
	</TabLink>
	<TabLink title={$t("common.suggested_emotes")} href="/emotes/{id}/suggested-emotes" responsive>
		<Graph />
		<Graph weight="fill" />
	</TabLink>
	<TabLink title={$t("common.mod_comments")} href="/emotes/{id}/mod-comments" responsive>
		<ChatText />
		<ChatText weight="fill" />
	</TabLink> -->
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
