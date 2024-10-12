<script lang="ts">
	import EmoteContainer from "$/components/layout/emote-container.svelte";
	import EmotePreview from "$/components/emote-preview.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.bookmarked_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<!-- <EmoteContainer scrollable>
	{#each Array(100) as _, i}
		<EmotePreview index={i} name="bookmarkedEmote{i}" />
	{/each}
</EmoteContainer> -->
