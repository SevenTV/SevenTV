<script lang="ts">
	import type { Snippet } from "svelte";
	import { t } from "svelte-i18n";

	let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
	<title>{$t("page_titles.editor_settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<section>
	<div>
		<h2>{$t("common.editors")}</h2>
		<span class="details">{$t("pages.settings.editors.details")}</span>
	</div>
	<div class="content">
		{@render children()}
	</div>
</section>

<style lang="scss">
	@use "../../../styles/settings.scss";
</style>
