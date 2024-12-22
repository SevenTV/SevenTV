<script lang="ts">
	import { graphql } from "$/gql";
	import type { User, UserSearchResult } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import type { ComponentProps, Snippet } from "svelte";
	import ChannelPreview from "./channel-preview.svelte";
	import TextInput from "./input/text-input.svelte";
	import Spinner from "./spinner.svelte";

	type Props = {
		onResultClick?: (e: MouseEvent, user: User) => void;
		icon?: Snippet;
		popup?: boolean;
	} & ComponentProps<typeof TextInput>;

	let { onResultClick, icon: providedIcon, popup = false, ...restProps }: Props = $props();

	let query = $state("");

	let timeout: NodeJS.Timeout | number | undefined; // not reactive

	async function search(query: string): Promise<UserSearchResult> {
		if (!query) {
			return { items: [], totalCount: 0, pageCount: 0 };
		}

		// Small timeout to prevent spamming requests when user is typing

		return new Promise((resolve, reject) => {
			if (timeout) {
				clearTimeout(timeout);
			}

			timeout = setTimeout(async () => {
				const res = await gqlClient()
					.query(
						graphql(`
							query EditorSearch($query: String!) {
								users {
									search(query: $query, page: 1, perPage: 5) {
										items {
											id
											mainConnection {
												platformDisplayName
												platformAvatarUrl
											}
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
										totalCount
										pageCount
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

				resolve(res.data.users.search as UserSearchResult);
			}, 200);
		});
	}

	let results = $derived(search(query));
</script>

<TextInput type="text" bind:value={query} {...restProps}>
	{#snippet icon()}
		{#await results}
			<Spinner />
		{:then _}
			{@render providedIcon?.()}
		{/await}
	{/snippet}
	{#snippet nonLabelChildren()}
		{#if popup}
			{#await results then results}
				{#if results.items.length > 0}
					<div class="popup-results">
						{#each results.items as result}
							<ChannelPreview user={result} size={2} onclick={(e) => onResultClick?.(e, result)} />
						{/each}
					</div>
				{/if}
			{/await}
		{/if}
	{/snippet}
</TextInput>
{#if !popup}
	{#await results then results}
		{#if results.items.length > 0}
			<div class="results">
				{#each results.items as result}
					<ChannelPreview user={result} size={2} onclick={(e) => onResultClick?.(e, result)} />
				{/each}
			</div>
		{/if}
	{/await}
{/if}

<style lang="scss">
	:global(label.input:has(input:enabled)):focus-within > .popup-results {
		display: flex;
	}

	.popup-results {
		position: absolute;
		top: calc(100% + 0.25rem);
		left: 0;
		right: 0;
		z-index: 10;

		background-color: var(--bg-light);

		border: 1px solid var(--border-active);
		border-radius: 0.5rem;

		display: none;
		overflow: hidden;

		flex-direction: column;

		& > :global(.button) {
			animation: expand-down 0.2s forwards;
		}
	}

	.results {
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
