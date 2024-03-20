<script lang="ts">
	import MonthlyPaints from "$/components/store/monthly-paints.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("pages.admin.overview")} - {$t("pages.admin.page_title_suffix")}</title>
</svelte:head>

<div class="grid">
	<MonthlyPaints />
</div>

<style lang="scss">
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(30rem, 1fr));
		gap: 1rem;
	}
</style>
