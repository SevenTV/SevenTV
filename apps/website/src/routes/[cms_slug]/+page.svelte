<script lang="ts">
	import type { PageData } from "./$types";
	import { t } from "svelte-i18n";

	let { data }: { data: PageData } = $props();
</script>

<svelte:head>
	<title>{data.slug} - {$t("page_titles.suffix")}</title>
</svelte:head>

<h1>CMS: {data.slug}</h1>
