<script lang="ts">
	import { t } from "svelte-i18n";
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
		<slot />
	</div>
</section>

<style lang="scss">
	@import "../../../styles/settings.scss";
</style>
