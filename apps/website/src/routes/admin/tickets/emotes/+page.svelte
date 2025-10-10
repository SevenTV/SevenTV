<script lang="ts">
	import EmoteTickets from "$/components/admin/emote-tickets.svelte";
	import { t } from "svelte-i18n";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { totalPublicRequests, countriesFilter, refetchRequested } from "$/lib/tickets";
	import type { ModRequestMessage } from "$/gql/graphql";
	import NotificationDialog from "$/components/dialogs/notification-dialog.svelte";
	let selectedMap = $state<ModRequestMessage[]>([]);
	let totalRequests = $state<number>(0);

	async function queryEmoteRequests(page: number, limit: number, wish: string, country: string[]) {
		const result = await gqlClient()
			.query(
				graphql(`
					query GetModRequests($page: Int!, $limit: Int!, $wish: String!, $country: [String!]!) {
						modRequests(page: $page, limit: $limit, wish: $wish, country: $country) {
							total
							messages {
								id
								kind
								created_at
								author_id
								target_id
								target_kind
								read
								wish
								actor_country_name
								actor_country_code
							}
						}
					}
				`),
				{ page, limit, wish, country },
				{
					url: "https://7tv.io/v3/gql",
					requestPolicy: "cache-and-network",
				},
			)
			.toPromise();

		if (result.error || !result.data || !result.data.modRequests) {
			throw result.error;
		}
		const output = result.data;
		selectedMap = result.data.modRequests.messages;
		totalRequests = output.modRequests.total;
		$totalPublicRequests = totalRequests;

		return {
			output,
		};
	}
	$effect(() => {
		if ($refetchRequested == true) {
			selectedMap = [];
			queryEmoteRequests(0, 250, "", $countriesFilter);
			$refetchRequested = false;
		}
		selectedMap = [];
		queryEmoteRequests(0, 250, "", $countriesFilter);
	});
</script>

<svelte:head>
	<title>{$t("page_titles.admin_emote_public_listing")} {$t("page_titles.admin_suffix")}</title>
</svelte:head>
<NotificationDialog />
<EmoteTickets bind:selectedMap />
