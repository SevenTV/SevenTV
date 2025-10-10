<script lang="ts">
	import type { Snippet } from "svelte";
	import { t } from "svelte-i18n";

	let { children }: { children: Snippet } = $props();
</script>

<svelte:head>
	<title>{$t("pages.admin.users.title")} - {$t("page_titles.admin_suffix")}</title>
</svelte:head>

{@render children()}
