<script lang="ts">
	import CmsPagePreview from "$/components/cms-page-preview.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("common.news")} - 7TV</title>
</svelte:head>

<div class="scroll-container">
	<div class="previews">
		{#each Array(10) as _, i}
			<CmsPagePreview slug="post{i}" />
		{/each}
	</div>
</div>

<style lang="scss">
	.scroll-container {
		overflow: auto;
		overflow: overlay;
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
