<script lang="ts">
	import EmoteTickets from "$/components/admin/emote-tickets.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.admin_emote_public_listing")} - {$t("page_titles.admin_suffix")}</title>
</svelte:head>

<EmoteTickets />
