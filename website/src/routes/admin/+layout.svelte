<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { Flag, PaintBrush, Table } from "phosphor-svelte";
	import { t } from "svelte-i18n";
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.admin.title")}</h1>
		<nav class="link-list">
			<TabLink title={$t("pages.admin.overview")} href="/admin">
				<Table />
				<Table weight="fill" slot="active" />
			</TabLink>
			<TabLink title={$t("pages.admin.reports")} href="/admin/reports">
				<Flag />
				<Flag weight="fill" slot="active" />
			</TabLink>
			<TabLink title={$t("common.cosmetics")} href="/admin/cosmetics">
				<PaintBrush />
				<PaintBrush weight="fill" slot="active" />
			</TabLink>
		</nav>
		<img src="/modge.webp" width="64" height="64" alt="Modge" class="modge" />
	</aside>
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.modge {
		margin-left: auto;
		margin-top: auto;
		filter: saturate(0) opacity(0.2);
	}
</style>
