<script lang="ts">
	import Banner from "$/components/store/admin-banner.svelte";

	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("pages.admin.overview")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<Banner title={$t("pages.admin.welcome")} subtitle={$t("pages.admin.enjoy")} gradientColor="#ee484b"></Banner>
