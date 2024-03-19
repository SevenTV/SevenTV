<script lang="ts">
	import { numberFormat } from "$/lib/utils";
	import TabLink from "../tab-link.svelte";
	import { Users, ChartLineUp, Graph, ChatText, Pulse } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	export let id: string;
</script>

<nav class="links">
	<TabLink
		title="{$t('common.channels', { values: { count: 2 } })} ({numberFormat().format(1020)})"
		href="/emotes/{id}"
		responsive
	>
		<Users />
		<Users weight="fill" slot="active" />
	</TabLink>
	<TabLink title={$t("common.activity")} href="/emotes/{id}/activity" responsive>
		<Pulse />
		<Pulse weight="fill" slot="active" />
	</TabLink>
	<TabLink title={$t("common.statistics")} href="/emotes/{id}/statistics" responsive>
		<ChartLineUp />
		<ChartLineUp weight="fill" slot="active" />
	</TabLink>
	<TabLink title={$t("common.suggested_emotes")} href="/emotes/{id}/suggested-emotes" responsive>
		<Graph />
		<Graph weight="fill" slot="active" />
	</TabLink>
	<TabLink title={$t("common.mod_comments")} href="/emotes/{id}/mod-comments" responsive>
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
