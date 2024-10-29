<script lang="ts">
	import Toggle from "$/components/input/toggle.svelte";
	import { t } from "svelte-i18n";
</script>

<svelte:head>
	<title>{$t("page_titles.notification_settings")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<section>
	<div>
		<h2>{$t("common.notifications")}</h2>
		<span class="details">{$t("pages.settings.notifications.details")}</span>
	</div>
	<div class="content">
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
		<hr />
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
		<hr />
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
	</div>
	<div class="content">
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
		<hr />
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
		<hr />
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
	</div>
	<div class="content">
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
		<hr />
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
		<hr />
		<div class="notification">
			<Toggle style="align-items: center">
				<div slot="left-label" class="label">
					<h3>Notification</h3>
					<span class="details">Lorem ipsum dolor, sit amet consectetur adipisicing elit.</span>
				</div>
			</Toggle>
		</div>
	</div>
</section>

<style lang="scss">
	@import "../../../styles/settings.scss";

	h3 {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.label {
		margin-right: auto;
	}
</style>
