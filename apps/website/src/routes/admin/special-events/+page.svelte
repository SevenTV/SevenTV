<script lang="ts">
	import AdminAssignEntitlementDialog from "$/components/dialogs/admin-assign-entitlement-dialog.svelte";
	import AdminCreateSpecialEventDialog from "$/components/dialogs/admin-create-special-event-dialog.svelte";
	import AdminViewEntitlementDialog from "$/components/dialogs/admin-view-entitlement-dialog.svelte";
	import type { DialogMode } from "$/components/dialogs/dialog.svelte";
	import Button from "$/components/input/button.svelte";
	import Spinner from "$/components/spinner.svelte";
	import UserName from "$/components/user-name.svelte";
	import { graphql } from "$/gql";
	import { EntitlementNodeTypeInput, type User } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import moment from "moment";
	import { Eye, Graph, Plus } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	async function querySpecialEvents() {
		const res = await gqlClient().query(
			graphql(`
				query AdminSpecialEvents {
					specialEvents {
						specialEvents {
							id
							name
							description
							tags
							createdBy {
								id
								mainConnection {
									platformDisplayName
								}
								style {
									activePaint {
										id
										name
										data {
											layers {
												id
												ty {
													__typename
													... on PaintLayerTypeSingleColor {
														color {
															hex
														}
													}
													... on PaintLayerTypeLinearGradient {
														angle
														repeating
														stops {
															at
															color {
																hex
															}
														}
													}
													... on PaintLayerTypeRadialGradient {
														repeating
														stops {
															at
															color {
																hex
															}
														}
														shape
													}
													... on PaintLayerTypeImage {
														images {
															url
															mime
															size
															scale
															width
															height
															frameCount
														}
													}
												}
												opacity
											}
											shadows {
												color {
													hex
												}
												offsetX
												offsetY
												blur
											}
										}
									}
								}
								highestRoleColor {
									hex
								}
							}
							createdAt
						}
					}
				}
			`),
			{},
		);

		return res.data?.specialEvents.specialEvents;
	}

	let results = $state<ReturnType<typeof querySpecialEvents>>();

	let createDialogMode: DialogMode = $state("hidden");

	$effect(() => {
		if (createDialogMode === "hidden") {
			results = querySpecialEvents();
		}
	});

	let assignEntitlementsDialog = $state<string>();
	let viewEntitlementsDialog = $state<string>();
</script>

<svelte:head>
	<title>{$t("pages.admin.special_events.header")} - {$t("page_titles.admin_suffix")}</title>
</svelte:head>

<div class="layout">
	<AdminCreateSpecialEventDialog bind:mode={createDialogMode} />
	<div class="buttons">
		<div class="buttons">
			<Button primary onclick={() => (createDialogMode = "shown")}>
				{#snippet icon()}
					<Plus />
				{/snippet}
				{$t("pages.admin.special_events.add")}
			</Button>
		</div>
		<div class="buttons">
			{#await results then results}
				{#if results}
					<p>{$t("pages.admin.special_events.results", { values: { "results.length": results.length }})}</p>
				{/if}
			{/await}
		</div>
	</div>
	<div class="table-wrapper">
		<table>
			<thead>
				<tr>
					<th>{$t("pages.admin.special_events.attributes.name")}</th>
					<th>{$t("pages.admin.special_events.attributes.description")}</th>
					<th>{$t("pages.admin.special_events.attributes.tags")}</th>
					<th>{$t("pages.admin.special_events.attributes.created_by")}</th>
					<th>{$t("pages.admin.special_events.attributes.created_at")}</th>
					<th>{$t("pages.admin.special_events.attributes.actions")}</th>
				</tr>
			</thead>
			<tbody>
				{#await results}
					<tr>
						<td colspan="6" style="text-align: center;">
							<Spinner />
						</td>
					</tr>
				{:then results}
					{#if results && results.length > 0}
						{#each results as specialEvent}
							{@const createdAt = moment(specialEvent.createdAt)}
							<AdminAssignEntitlementDialog
								bind:mode={() =>
									assignEntitlementsDialog === specialEvent.id ? "shown" : "hidden",
								(mode) => {
									if (mode === "hidden") {
										assignEntitlementsDialog = undefined;
									}
								}}
								from={{ type: EntitlementNodeTypeInput.SpecialEvent, id: specialEvent.id }}
								fromName={specialEvent.name}
							/>
							<AdminViewEntitlementDialog
								bind:mode={() => (viewEntitlementsDialog === specialEvent.id ? "shown" : "hidden"),
								(mode) => {
									if (mode === "hidden") {
										viewEntitlementsDialog = undefined;
									}
								}}
								from={{ type: EntitlementNodeTypeInput.SpecialEvent, id: specialEvent.id }}
								fromName={specialEvent.name}
							/>
							<tr>
								<td>{specialEvent.name}</td>
								<td>{specialEvent.description}</td>
								<td>
									{specialEvent.tags.join(", ")}
								</td>
								<td>
									{#if specialEvent.createdBy}
										<UserName user={specialEvent.createdBy as User} />
									{:else}
										{$t("pages.admin.special_events.system")}
									{/if}
								</td>
								<td>
									{createdAt.format()}
									<br />
									{createdAt.fromNow()}
								</td>
								<td>
									<div class="actions">
										<Button
											title="Assign Entitlements"
											onclick={() => (assignEntitlementsDialog = specialEvent.id)}
										>
											{#snippet icon()}
												<Graph />
											{/snippet}
										</Button>
										<Button
											title="View Entitlements"
											onclick={() => (viewEntitlementsDialog = specialEvent.id)}
										>
											{#snippet icon()}
												<Eye />
											{/snippet}
										</Button>
									</div>
								</td>
							</tr>
						{/each}
					{:else}
						<tr>
							<td colspan="6" style="text-align: center;">{$t("pages.admin.special_events.no_events")}</td>
						</tr>
					{/if}
				{/await}
			</tbody>
		</table>
	</div>
</div>

<style lang="scss">
	.layout {
		max-width: 100rem;
		margin: 0 auto;
		width: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		max-height: 100%;
	}

	.buttons {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.actions {
		display: flex;
		align-items: center;
	}

	.table-wrapper {
		overflow: auto;
	}
</style>
