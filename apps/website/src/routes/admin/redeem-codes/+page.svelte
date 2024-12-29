<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import Spinner from "$/components/spinner.svelte";
	import UserName from "$/components/user-name.svelte";
	import { graphql } from "$/gql";
	import type { RedeemCodeSearchResult, User } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import moment from "moment";
	import { CaretLeft, CaretRight, MagnifyingGlass, Plus, Trash } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	let query = $state("");

	let page: number | undefined = $state(1);

	const PAGE_MIN_LIMIT = 1;
	const PAGE_MAX_LIMIT = 250;

	function decreasePage() {
		if (!page) {
			page = 1;
			return;
		}

		if (page && page > PAGE_MIN_LIMIT) page--;
	}

	function increasePage() {
		if (!page) {
			page = 1;
			return;
		}

		if (page < PAGE_MAX_LIMIT) page++;
	}

	let remainingUses = $state(false);

	async function queryCodes(query: string, remainingUses: boolean, page?: number) {
		const res = await gqlClient().query(
			graphql(`
				query AdminRedeemCodes($query: String, $remainingUses: Boolean, $page: Int) {
					redeemCodes {
						redeemCodes(query: $query, remainingUses: $remainingUses, page: $page, perPage: 100) {
							items {
								id
								name
								description
								tags
								code
								remainingUses
								activePeriod {
									start
									end
								}
								effect {
									__typename
									... on CodeEffectSpecialEvent {
										specialEvent {
											id
											name
										}
									}
									... on CodeEffectDirectEntitlement {
										entitlements {
											__typename
										}
									}
								}
								subscriptionEffect {
									id
									trialDays
								}
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
							totalCount
							pageCount
						}
					}
				}
			`),
			{ query, remainingUses, page },
		);

		return res.data?.redeemCodes.redeemCodes as RedeemCodeSearchResult;
	}

	let results = $state<Promise<RedeemCodeSearchResult>>();

	$effect(() => {
		results = queryCodes(query, remainingUses, page);
	});

	async function deactivateCode(id: string) {
		const res = await gqlClient().mutation(
			graphql(`
				mutation AdminDeactivateRedeemCode($id: Id!) {
					redeemCodes {
						redeemCode(id: $id) {
							deactivate {
								id
							}
						}
					}
				}
			`),
			{ id },
		);

		if (res.data?.redeemCodes.redeemCode.deactivate) {
			results = queryCodes(query, remainingUses, page);
		}
	}
</script>

<svelte:head>
	<title>Redeem Codes - {$t("page_titles.admin_suffix")}</title>
</svelte:head>

<div class="layout">
	<div class="buttons">
		<div class="buttons">
			<Button primary>
				{#snippet icon()}
					<Plus />
				{/snippet}
				Add Code(s)
			</Button>
			<TextInput placeholder="Search" bind:value={query}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput>
			<div>
				Filters:
				<Checkbox bind:value={remainingUses}>Remaining Uses</Checkbox>
			</div>
		</div>
		<div class="buttons">
			{#await results then results}
				{#if results}
					<p>Found {results.totalCount} results ({results.pageCount} pages)</p>
				{/if}
			{/await}
			<div class="buttons-horizontal">
				<Button secondary onclick={decreasePage}>
					{#snippet icon()}
						<CaretLeft />
					{/snippet}
				</Button>
				<input
					type="number"
					placeholder="Page"
					min={PAGE_MIN_LIMIT}
					max={PAGE_MAX_LIMIT}
					bind:value={page}
				/>
				<Button secondary onclick={increasePage}>
					{#snippet icon()}
						<CaretRight />
					{/snippet}
				</Button>
			</div>
		</div>
	</div>
	<div class="table-wrapper">
		<table>
			<thead>
				<tr>
					<th>Name</th>
					<th>Description</th>
					<th>Tags</th>
					<th>Code</th>
					<th>Remaining Uses</th>
					<th>Active Period</th>
					<th>Effect</th>
					<th>Created By</th>
					<th>Created At</th>
					<th>Actions</th>
				</tr>
			</thead>
			<tbody>
				{#await results}
					<tr>
						<td colspan="10" style="text-align: center;">
							<Spinner />
						</td>
					</tr>
				{:then results}
					{#if results && results.items.length > 0}
						{#each results.items as code}
							{@const createdAt = moment(code.createdAt)}
							<tr>
								<td>{code.name}</td>
								<td>{code.description}</td>
								<td>
									{code.tags.join(", ")}
								</td>
								<td>
									<code>{code.code}</code>
								</td>
								<td>{code.remainingUses}</td>
								<td>
									{#if code.activePeriod}
										{moment(code.activePeriod.start).format()} -
										{moment(code.activePeriod.end).format()}
									{:else}
										Unlimited
									{/if}
								</td>
								<td>
									{#if code.effect.__typename === "CodeEffectSpecialEvent"}
										{code.effect.specialEvent?.name ?? "Unknown Special Event"}
									{:else if code.effect.__typename === "CodeEffectDirectEntitlement"}
										{code.effect.entitlements
											.map((entitlement) => entitlement.__typename)
											.join(", ")}
									{/if}
								</td>
								<td>
									{#if code.createdBy}
										<UserName user={code.createdBy as User} />
									{:else}
										System
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
											disabled={code.remainingUses === 0}
											onclick={() => deactivateCode(code.id)}
											title="Deactivate"
										>
											{#snippet icon()}
												<Trash />
											{/snippet}
										</Button>
									</div>
								</td>
							</tr>
						{/each}
					{:else}
						<tr>
							<td colspan="10" style="text-align: center;">No Codes</td>
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

	.buttons-horizontal {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.actions {
		display: flex;
		align-items: center;
	}

	input[type="number"] {
		-moz-appearance: textfield;
		appearance: textfield;

		text-align: center;
		width: 3rem;
		height: 2rem;
		border: none;
		padding: 0.5rem;
		border-radius: 0.5rem;

		&::-webkit-outer-spin-button,
		&::-webkit-inner-spin-button {
			-webkit-appearance: none;
			margin: 0;
		}
	}

	.table-wrapper {
		overflow: auto;
	}
</style>
