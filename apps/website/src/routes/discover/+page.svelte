<script lang="ts">
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("common.news")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<div class="scroll-container">
	<div class="previews">
	</div>
</div>

<style lang="scss">
	.scroll-container {
		overflow: auto;
		scrollbar-gutter: stable;
	}

	.previews {
		max-width: 40rem;
		margin-inline: auto;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
