<script lang="ts">
	import { t } from "svelte-i18n";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import type { ReportStatus, Report } from "$/gql/graphql";
	import ReportTickets from "$/components/admin/report-tickets.svelte";
	import { totalReportRequests } from "$/lib/tickets";
	let selectedMap = $state<Report[]>([]);

	async function queryReports(status: ReportStatus, limit: number, page?: number) {
		const result = await gqlClient()
			.query(
				graphql(`
					query GetReports($status: ReportStatus, $limit: Int, $page: Int) {
						reports(status: $status, limit: $limit, page: $page) {
							id
							target_kind
							target_id
							actor {
								id
								username
								display_name
								avatar_url
								__typename
							}
							assignees {
								id
								username
								display_name
								avatar_url
								__typename
							}
							status
							subject
							body
							actor_id
							created_at
							notes
							priority
						}
					}
				`),
				{ status, limit, page },
				{
					url: "https://7tv.io/v3/gql",
				},
			)
			.toPromise();

		if (result.error || !result.data || !result.data) {
			throw result.error;
		}
		const output = result.data;
		selectedMap = output.reports;
		let totalRequests = output.reports.length;
		$totalReportRequests = totalRequests;

		return {
			output,
		};
	}
	$effect(() => {
		selectedMap = [];
		queryReports("OPEN" as ReportStatus, 250);
	});
</script>

<svelte:head>
	<title>{$t("pages.admin.tickets.reports")}</title>
</svelte:head>
<ReportTickets bind:selectedMap />
