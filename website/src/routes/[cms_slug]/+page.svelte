<script lang="ts">
	import type { PageData } from "./$types";
	import { t } from "svelte-i18n";

	export let data: PageData;
</script>

<svelte:head>
	<title>{data.slug} - {$t("page_titles.suffix")}</title>
</svelte:head>

<h1>{data.slug}</h1>
