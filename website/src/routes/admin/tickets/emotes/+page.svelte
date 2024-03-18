<script lang="ts">
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.admin_emote_tickets")} - {$t("pages.admin.page_title_suffix")}</title>
</svelte:head>

{$t("common.emotes", { values: { count: 2 } })}
