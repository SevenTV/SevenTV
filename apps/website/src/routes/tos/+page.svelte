<script lang="ts">
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>Terms of Service - {$t("page_titles.suffix")}</title>
</svelte:head>

<div class="page">
	<h1>{$t("pages.terms.title")}</h1>
	<p>{$t("pages.terms.last_updated")}</p>
	<p>
		{$t("pages.terms.description_1")}
	</p>
	<p>
		{$t("pages.terms.description_2")}
	</p>
	<p>
		{$t("pages.terms.description_3")}
	</p>
	<p>
		{$t("pages.terms.description_4")}
	</p>
	<h2>{$t("pages.terms.conditions.header")}</h2>
	<p>
		{$t("pages.terms.conditions.description")}
	</p>
	<h2>{$t("pages.terms.privacy.header")}</h2>
	<p>
		{$t("pages.terms.privacy.description")}
	</p>
	<h2>{$t("pages.terms.modifications.header")}</h2>
	<p>
		{$t("pages.terms.modifications.description")}
	</p>
	<h2>{$t("pages.terms.terms_conditions.header")}</h2>
	<p>
		{$t("pages.terms.terms_conditions.description_1")}
	</p>
	<p>{$t("pages.terms.terms_conditions.description_2")}</p>
	<p>
		{$t("pages.terms.terms_conditions.description_3")}
	</p>
	<p>
		{$t("pages.terms.terms_conditions.description_4")}
	</p>
	<h2>{$t("pages.terms.store.header")}</h2>
	<p>
		{$t("pages.terms.store.description_1")}
	</p>
	<p>
		{$t("pages.terms.store.description_2")}
	</p>
	<h2>{$t("pages.terms.accuracy_completeness_timelessness.header")}</h2>
	<p>
		{$t("pages.terms.accuracy_completeness_timelessness.description_1")}
	</p>
	<p>
		{$t("pages.terms.accuracy_completeness_timelessness.description_2")}
	</p>
	<h2>{$t("pages.terms.user_content.header")}</h2>
	<p>
		{$t("pages.terms.user_content.description")}
	</p>
	<h2>{$t("pages.terms.changes_amendments.header")}</h2>
	<p>
		{$t("pages.terms.changes_amendments.description")}
	</p>
	<h2>{$t("pages.terms.dmca.header")}</h2>
	<p>
		{$t("pages.terms.dmca.description_1")}
	</p>
	<p>
		{$t("pages.terms.dmca.description_2")}
	</p>
	<h2>{$t("pages.terms.copyright_complaint.header")}</h2>
	<p>
		{$t("pages.terms.copyright_complaint.description_1")}
	</p>
	<p>
		{$t("pages.terms.copyright_complaint.description_2")}
	</p>
	<p>
		{$t("pages.terms.copyright_complaint.description_3")}
	</p>
	<h2>{$t("pages.terms.infringement_notifications.header")}</h2>
	<p>
		{$t("pages.terms.infringement_notifications.description_1")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_2")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_3")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_4")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_5")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_6")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_7")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_8")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_9")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_10")}
	</p>
	<p>
		{$t("pages.terms.infringement_notifications.description_11")}
	</p>
	<h2>{$t("pages.terms.counter_notifications.header")}</h2>
	<p>
		{$t("pages.terms.counter_notifications.description_1")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_2")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_3")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_4")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_5")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_6")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_7")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_8")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_9")}
	</p>
	<p>
		{$t("pages.terms.counter_notifications.description_10")}
	</p>
	<h2>{$t("pages.terms.sale.header")}</h2>
	<h2>{$t("pages.terms.amendments_changes.header")}</h2>
	<p>
		{$t("pages.terms.amendments_changes.description")}
	</p>
	<h2>{$t("pages.terms.reporting.header")}</h2>
	<p>
		{$t("pages.terms.reporting.description")} <a href="mailto:kathy@7tv.app">kathy@7tv.app</a>.
	</p>
</div>

<style lang="scss">
	.page {
		max-width: 60rem;
		margin: 2rem auto;
		padding: 0 1rem;
	}

	h1 {
		margin: 1.4rem 0;
	}

	h2 {
		margin: 1.2rem 0;
	}

	p {
		margin: 1rem 0;
	}
</style>
