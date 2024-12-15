<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";
	import Spinner from "../spinner.svelte";
	import TextInput from "../input/text-input.svelte";
	import { CaretLeft, User as UserIcon } from "phosphor-svelte";
	import {
		SubscriptionProductKind,
		type SubscriptionProductVariant,
		type User,
	} from "$/gql/graphql";
	import UserName from "../user-name.svelte";
	import ChannelPreview from "../channel-preview.svelte";
	import { variantUnit } from "$/lib/utils";

	interface Props {
		mode: DialogMode;
		variant: SubscriptionProductVariant;
	}

	let { mode = $bindable("hidden"), variant }: Props = $props();

	let recipient = $state<User>();
	let query = $state("");

	let timeout: NodeJS.Timeout | number | undefined; // not reactive

	async function search(query: string): Promise<User[]> {
		if (!query) {
			return [];
		}

		if (timeout) {
			clearTimeout(timeout);
		}

		// Small timeout to prevent spamming requests when user is typing

		return new Promise((resolve, reject) => {
			timeout = setTimeout(async () => {
				const res = await gqlClient()
					.query(
						graphql(`
							query SearchUser($query: String!) {
								users {
									search(query: $query, page: 1, perPage: 10) {
										items {
											id
											mainConnection {
												platformDisplayName
												platformAvatarUrl
											}
											roleIds
											style {
												activeProfilePicture {
													images {
														url
														mime
														size
														width
														height
														scale
														frameCount
													}
												}
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
						`),
						{ query },
					)
					.toPromise();

				if (res.error || !res.data) {
					reject();
					return;
				}

				resolve(res.data.users.search.items as User[]);
			}, 200);
		});
	}

	let results = $derived(search(query));

	let giftLoading = $state(false);

	async function gift() {
		if (!recipient) {
			return;
		}

		giftLoading = true;

		const res = await gqlClient()
			.mutation(
				graphql(`
					mutation Subscribe($userId: Id!, $variantId: ProductId!) {
						billing(userId: $userId) {
							subscribe(variantId: $variantId) {
								checkoutUrl
							}
						}
					}
				`),
				{ userId: recipient.id, variantId: variant.id },
			)
			.toPromise();

		if (res.data) {
			window.location.href = res.data.billing.subscribe.checkoutUrl;
		}

		giftLoading = false;
	}
</script>

<Dialog bind:mode>
	<form class="layout">
		<h1>Gift 1 {variantUnit(variant)}</h1>
		<hr />
		{#if recipient}
			<p>
				Gift 1 {variantUnit(variant)} to <UserName user={recipient} />.
			</p>
		{:else}
			<TextInput type="text" placeholder="Search User" bind:value={query}>
				{#snippet icon()}
					{#await results}
						<Spinner />
					{:then _}
						<UserIcon />
					{/await}
				{/snippet}
				<h2>Select recipient</h2>
			</TextInput>
			{#await results then results}
				{#if results && results.length > 0}
					<div class="results">
						{#each results as result}
							<ChannelPreview
								user={result}
								size={2}
								onclick={(e) => {
									e.preventDefault();
									recipient = result;
								}}
							/>
						{/each}
					</div>
				{/if}
			{/await}
		{/if}

		<div class="buttons">
			{#if recipient}
				<Button secondary onclick={() => (recipient = undefined)} style="margin-right: auto">
					{#snippet icon()}
						<CaretLeft />
					{/snippet}
					Back
				</Button>
			{/if}

			{#if !recipient}
				<Button secondary onclick={() => (mode = "hidden")} style="margin-left: auto"
					>{$t("labels.cancel")}</Button
				>
			{/if}

			{#snippet spinnerIcon()}
				<Spinner />
			{/snippet}

			<Button
				icon={giftLoading ? spinnerIcon : undefined}
				disabled={giftLoading || !recipient}
				onclick={gift}
				primary
				submit
			>
				Continue
			</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		height: 100%;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	h2 {
		font-size: 1rem;
		font-weight: 400;
	}

	.buttons {
		margin-top: auto;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.results {
		background-color: var(--bg-light);

		border: 1px solid var(--border-active);
		border-radius: 0.5rem;

		display: flex;
		overflow: hidden;

		flex-direction: column;

		& > :global(.button) {
			animation: expand-down 0.2s forwards;
		}
	}

	@keyframes expand-down {
		from {
			height: 2rem;
		}
		to {
			height: 2.75rem;
		}
	}
</style>
