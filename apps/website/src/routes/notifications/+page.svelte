<script lang="ts">
	import Notifications from "$/components/notifications/notifications.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("common.notifications")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<Notifications />
