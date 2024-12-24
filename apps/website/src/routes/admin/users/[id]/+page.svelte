<script lang="ts">
	import moment from "moment";
	import type { PageData } from "./$types";
	import { Ulid, Uuid } from "id128";
	import Button from "$/components/input/button.svelte";
	import Role from "$/components/users/role.svelte";
	import SubInfo from "$/components/sub-info.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import Spinner from "$/components/spinner.svelte";
	import { SubscriptionProvider, type User } from "$/gql/graphql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";
	import { CaretLeft } from "phosphor-svelte";
	import UserName from "$/components/user-name.svelte";
	import { user } from "$/lib/auth";
	import AdminCreateSessionDialog from "$/components/dialogs/admin-create-session-dialog.svelte";
	import type { DialogMode } from "$/components/dialogs/dialog.svelte";

	let { data }: { data: PageData } = $props();

	let parsedId = $derived(Ulid.fromCanonicalTrusted(data.id));

	let idTime = $derived(moment(parsedId.time));

	let uuid = $derived(Uuid.fromRawTrusted(parsedId.toRaw()).toCanonical());

	async function queryUser(id: string, showAllPeriods: boolean) {
		const res = await gqlClient().query(
			graphql(`
				query AdminGetUser($id: Id!, $productId: Id!, $showAllPeriods: Boolean!) {
					users {
						user(id: $id) {
							id
							mainConnection {
								platformDisplayName
							}
							roles {
								name
								color {
									hex
								}
							}
							stripeCustomerId
							updatedAt
							searchUpdatedAt
							billing(productId: $productId) {
								subscriptionInfo {
									totalDays
									endDate
									activePeriod {
										subscriptionProductVariant {
											kind
										}
										subscription {
											state
										}
										autoRenew
										end
										giftedBy {
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
									}
									periods @include(if: $showAllPeriods) {
										id
										providerId {
											provider
											id
										}
										productId
										start
										end
										isTrial
										autoRenew
										giftedBy {
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
										createdBy {
											__typename
											... on SubscriptionPeriodCreatedByRedeemCode {
												redeemCodeId
											}
											... on SubscriptionPeriodCreatedByInvoice {
												invoiceId
											}
											... on SubscriptionPeriodCreatedBySystem {
												reason
											}
										}
										subscriptionProductVariant {
											kind
										}
									}
								}
							}
						}
					}
				}
			`),
			{ id, productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID, showAllPeriods },
		);

		return res.data?.users?.user as User | undefined;
	}

	let userData = $derived(queryUser(data.id, !!$user?.permissions.user.manageBilling));

	let canManageSessions = $derived($user?.permissions.user.manageSessions);

	let deleteAllSessionsLoading = $state(false);

	async function deleteAllSessions(userId: string) {
		deleteAllSessionsLoading = true;

		await gqlClient().mutation(
			graphql(`
				mutation AdminDeleteAllSessions($userId: Id!) {
					users {
						user(id: $userId) {
							deleteAllSessions
						}
					}
				}
			`),
			{ userId },
		);

		deleteAllSessionsLoading = false;
	}

	let createSessionDialogMode = $state<DialogMode>("hidden");
</script>

<div class="layout">
	<div class="buttons">
		<Button href="/admin/users" secondary>
			{#snippet icon()}
				<CaretLeft />
			{/snippet}
			Back
		</Button>
	</div>
	{#await userData}
		<Spinner />
	{:then user}
		{#if user}
			{@const updatedAt = moment(user.updatedAt)}
			{@const searchUpdatedAt = user.searchUpdatedAt ? moment(user.searchUpdatedAt) : undefined}
			<h1>User: {user.mainConnection?.platformDisplayName}</h1>
			<div class="content">
				<section>
					<h2>Details</h2>
					<table>
						<thead>
							<tr>
								<th>Key</th>
								<th>Value</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td>ID</td>
								<td>
									<code>{data.id}</code>
									<Button
										secondary
										style="display:inline-block"
										onclick={() => navigator.clipboard.writeText(data.id)}>Copy ULID</Button
									>
									<Button
										secondary
										style="display:inline-block"
										onclick={() => navigator.clipboard.writeText(uuid)}>Copy UUID</Button
									>
								</td>
							</tr>
							<tr>
								<td>ID (Timestamp)</td>
								<td>
									<code>{idTime.toISOString()}</code>
									<br />
									{idTime.fromNow()}
								</td>
							</tr>
							<tr>
								<td>Last Updated At</td>
								<td>
									<code>{updatedAt.toISOString()}</code>
									<br />
									{updatedAt.fromNow()}
								</td>
							</tr>
							<tr>
								<td>Search Last Updated At</td>
								<td>
									{#if searchUpdatedAt}
										<code>{searchUpdatedAt.toISOString()}</code>
										<br />
										{searchUpdatedAt.fromNow()}
									{:else}
										Queued for update
									{/if}
								</td>
							</tr>
							<tr>
								<td>Stripe Customer</td>
								<td>
									{#if user.stripeCustomerId}
										<code>{user.stripeCustomerId}</code>
										<Button
											href="https://dashboard.stripe.com/customers/{user.stripeCustomerId}"
											target="_blank"
											secondary
											style="display:inline-block">View</Button
										>
									{:else}
										No stripe customer associated
									{/if}
								</td>
							</tr>
							<tr>
								<td>Roles</td>
								<td>
									<div class="roles">
										{#each user.roles as role}
											<Role roleData={role} />
										{/each}
									</div>
								</td>
							</tr>
						</tbody>
					</table>
				</section>
				{#if canManageSessions}
					<AdminCreateSessionDialog bind:mode={createSessionDialogMode} userId={data.id} />
					<section>
						<h2>Actions</h2>
						<div class="action-group">
							<h3>Sessions</h3>
							{#snippet loadingSpinner()}
								<Spinner />
							{/snippet}
							<div class="buttons">
								<Button secondary onclick={() => (createSessionDialogMode = "shown")}>Create New Session</Button>
								<Button
									secondary
									style="color:var(--danger)"
									disabled={deleteAllSessionsLoading}
									onclick={() => deleteAllSessions(data.id)}
									icon={deleteAllSessionsLoading ? loadingSpinner : undefined}
								>
									Delete All Sessions
								</Button>
							</div>
						</div>
						<!-- <div class="action-group">
							<h3>Subscription</h3>
							<div class="buttons">
								<Button secondary style="color:var(--danger)">Cancel</Button>
								</div>
								</div> -->
					</section>
				{/if}
				<section>
					<h2>Subscription</h2>
					<SubInfo
						data={user.billing.subscriptionInfo}
						style="background-color:var(--bg-light); border-radius: 0.5rem;"
					/>
					{#if user.billing.subscriptionInfo.periods}
						<h2>Subscription Periods ({user.billing.subscriptionInfo.periods.length})</h2>
						<div class="periods">
							{#each user.billing.subscriptionInfo.periods.toReversed() as period}
								{@const periodId = Ulid.fromCanonicalTrusted(period.id)}
								{@const periodUuid = Uuid.fromRawTrusted(periodId.toRaw()).toCanonical()}
								{@const periodTime = moment(periodId.time)}
								<div class="period">
									<table>
										<thead>
											<tr>
												<th>Key</th>
												<th>Value</th>
											</tr>
										</thead>
										<tbody>
											<tr>
												<td>ID</td>
												<td>
													<code>{period.id}</code>
													<Button
														secondary
														style="display:inline-block"
														onclick={() => navigator.clipboard.writeText(period.id)}
														>Copy ULID</Button
													>
													<Button
														secondary
														style="display:inline-block"
														onclick={() => navigator.clipboard.writeText(periodUuid)}
														>Copy UUID</Button
													>
												</td>
											</tr>
											<tr>
												<td>ID (Timestamp)</td>
												<td>
													<code>
														{periodTime.toISOString()}
													</code>
													<br />
													{periodTime.fromNow()}
												</td>
											</tr>
											{#if period.providerId}
												<tr>
													<td>Provider ID</td>
													<td>
														{period.providerId.provider} - <code>{period.providerId.id}</code>
														{#if period.providerId.provider === SubscriptionProvider.Stripe}
															<Button
																href="https://dashboard.stripe.com/subscriptions/{period.providerId
																	.id}"
																target="_blank"
																secondary
																style="display:inline-block">View</Button
															>
														{/if}
													</td>
												</tr>
											{/if}
											<tr>
												<td>Product ID</td>
												<td>
													<code>{period.productId}</code>
													<Button
														href="https://dashboard.stripe.com/prices/{period.productId}"
														target="_blank"
														secondary
														style="display:inline-block">View</Button
													>
												</td>
											</tr>
											<tr>
												<td>Start</td>
												<td>
													<code>{moment(period.start).toISOString()}</code>
													<br />
													{moment(period.start).fromNow()}
												</td>
											</tr>
											<tr>
												<td>End</td>
												<td>
													<code>{moment(period.end).toISOString()}</code>
													<br />
													{moment(period.end).fromNow()}
												</td>
											</tr>
											<tr>
												<td>Is Trial?</td>
												<td>
													{period.isTrial ? "Yes" : "No"}
												</td>
											</tr>
											<tr>
												<td>Auto Renew?</td>
												<td>
													{period.autoRenew ? "Yes" : "No"}
												</td>
											</tr>
											{#if period.giftedBy}
												<tr>
													<td>Gifted By</td>
													<td>
														<a href="/users/{period.giftedBy.id}">
															<UserName user={period.giftedBy} />
														</a>
													</td>
												</tr>
											{/if}
											<tr>
												<td>Created By</td>
												<td>
													{#if period.createdBy.__typename === "SubscriptionPeriodCreatedByRedeemCode"}
														Redeem Code - <code>{period.createdBy.redeemCodeId}</code>
													{:else if period.createdBy.__typename === "SubscriptionPeriodCreatedByInvoice"}
														Invoice - <code>{period.createdBy.invoiceId}</code>
														<Button
															href="https://dashboard.stripe.com/invoices/{period.createdBy
																.invoiceId}"
															target="_blank"
															secondary
															style="display:inline-block">View</Button
														>
													{:else if period.createdBy.__typename === "SubscriptionPeriodCreatedBySystem"}
														System - {period.createdBy.reason}
													{/if}
												</td>
											</tr>
											<tr>
												<td>Product Variant</td>
												<td>
													{period.subscriptionProductVariant.kind}
												</td>
											</tr>
										</tbody>
									</table>
								</div>
							{/each}
						</div>
					{/if}
				</section>
			</div>
		{:else}
			<p>User not found</p>
		{/if}
	{/await}
</div>

<style lang="scss">
	.layout {
		margin-inline: auto;
		width: 100%;
		max-width: 100rem;

		overflow: auto;
		scrollbar-gutter: stable;
	}

	.content {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
	}

	section {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
	}

	.roles {
		display: flex;
		gap: 0.5rem;
	}

	h2 {
		margin-top: 0.5rem;
	}

	.periods {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(25rem, 1fr));
		gap: 1rem;
	}

	.action-group {
		border: 2px solid var(--border-active);
		border-radius: 0.5rem;
		padding: 1rem;

		h3 {
			margin-bottom: 0.5rem;
		}
	}
</style>
