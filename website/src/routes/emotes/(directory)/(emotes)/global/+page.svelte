<script lang="ts">
	import EmoteLoadingPlaceholder from "$/components/emote-loading-placeholder.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.global_emotes")} - {$t("page_titles.suffix")}</title>
</svelte:head>

{#each Array(30) as _, i}
	<EmoteLoadingPlaceholder index={i} />
{/each}
