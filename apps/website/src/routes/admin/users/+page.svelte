<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import Spinner from "$/components/spinner.svelte";
	import UserSearch from "$/components/user-search.svelte";
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import { User } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	let rerunJobLoading = $state(false);

	async function rerunSubscriptionBenefitsJob() {
		rerunJobLoading = true;

		const res = await gqlClient().mutation(
			graphql(`
				mutation RerunSubscriptionRefreshJob {
					jobs {
						rerunSubscriptionRefreshJob
					}
				}
			`),
			{},
		);

		rerunJobLoading = false;

		return res.data?.jobs.rerunSubscriptionRefreshJob;
	}
</script>

<div class="layout">
	<div class="action-group">
		<h2>{$t("pages.admin.users.manage")}</h2>
		<UserSearch
			searchlimit={15}
			placeholder={$t("pages.admin.users.search_user_placeholder")}
			resulthref={(user) => `/admin/users/${user.id}`}
		>
			<p>{$t("pages.admin.users.manage_single_user")}</p>
			{#snippet icon()}
				<User />
			{/snippet}
		</UserSearch>
	</div>
	<div class="action-group">
		<h2>{$t("pages.admin.users.actions")}</h2>
		<div class="action">
			<h3>{$t("pages.admin.users.subscription_benefits_job")}</h3>
			<p>
				{$t("pages.admin.users.subscription_benefits_job_desc")}
			</p>
			{#snippet loadingSpinner()}
				<Spinner />
			{/snippet}
			<Button
				secondary
				disabled={rerunJobLoading}
				onclick={rerunSubscriptionBenefitsJob}
				icon={rerunJobLoading ? loadingSpinner : undefined}
			>
				{$t("pages.admin.users.rerun")}
			</Button>
		</div>
	</div>
</div>

<style lang="scss">
	.layout {
		display: flex;
		justify-content: center;
		align-items: start;
		gap: 1rem;
		flex-wrap: wrap;
	}

	.action-group {
		flex-grow: 1;
		max-width: 40rem;

		border: 2px solid var(--border-active);
		border-radius: 0.5rem;
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		.action {
			display: flex;
			flex-direction: column;
			align-items: start;
			gap: 0.25rem;
		}
	}
</style>
