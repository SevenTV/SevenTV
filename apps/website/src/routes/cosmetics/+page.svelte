<script lang="ts">
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("common.cosmetics")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<h1>{$t("common.cosmetics")}</h1>

<div class="paints">
</div>

<style lang="scss">
	.paints {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 1rem;
	}
</style>
