<script lang="ts">
	import moment from "moment";
	import type { PageData } from "./$types";
	import { Ulid, Uuid4 } from "id128";
	import Button from "$/components/input/button.svelte";
	import Role from "$/components/users/role.svelte";
	import SubInfo from "$/components/sub-info.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import Spinner from "$/components/spinner.svelte";
	import type { User } from "$/gql/graphql";
	import { PUBLIC_SUBSCRIPTION_PRODUCT_ID } from "$env/static/public";

	let { data }: { data: PageData } = $props();

	let parsedId = $derived(Ulid.fromCanonicalTrusted(data.id));

	let idTime = $derived(moment(parsedId.time));

	let uuid = $derived(Uuid4.fromRawTrusted(parsedId.toRaw()).toCanonical());

	async function queryUser(id: string) {
		const res = await gqlClient().query(
			graphql(`
				query AdminGetUser($id: Id!, $productId: Id!) {
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
								}
							}
						}
					}
				}
			`),
			{ id, productId: PUBLIC_SUBSCRIPTION_PRODUCT_ID },
		);

		return res.data?.users?.user as User | undefined;
	}

	let user = $derived(queryUser(data.id));
</script>

{#await user}
	<Spinner />
{:then user}
	{#if user}
		{@const updatedAt = moment(user.updatedAt)}
		{@const searchUpdatedAt = user.searchUpdatedAt ? moment(user.searchUpdatedAt) : undefined}
		<div class="layout">
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
								<td>ID (ULID)</td>
								<td>
									<code>{data.id}</code>
									<Button secondary style="display:inline-block" onclick={() => navigator.clipboard.writeText(data.id)}>Copy</Button>
								</td>
							</tr>
							<tr>
								<td>ID (UUID)</td>
								<td>
									<code>{uuid}</code>
									<Button secondary style="display:inline-block" onclick={() => navigator.clipboard.writeText(uuid)}>Copy</Button>
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
					<h2>Subscription</h2>
					<SubInfo data={user.billing.subscriptionInfo} style="background-color:var(--bg-light); border-radius: 0.5rem;" />
				</section>
				<section>
					<h2>Actions</h2>
					<div class="action-group">
						<h3>Sessions</h3>
						<div class="buttons">
							<Button secondary>Create New Session</Button>
							<Button secondary style="color:var(--danger)">Delete All Sessions</Button>
						</div>
					</div>
					<div class="action-group">
						<h3>Subscription</h3>
						<div class="buttons">
							<Button secondary style="color:var(--danger)">Cancel</Button>
						</div>
					</div>
				</section>
			</div>
		</div>
	{:else}
		<p>User not found</p>
	{/if}
{/await}

<style lang="scss">
	.layout {
		margin-inline: auto;
		width: 100%;
		max-width: 100rem;
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

	.action-group {
		border: 2px solid var(--border-active);
		border-radius: 0.5rem;
		padding: 1rem;

		h3 {
			margin-bottom: 0.5rem;
		}
	}
</style>
