<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import { CaretLeft, CaretRight, MagnifyingGlass } from "phosphor-svelte";
	import type { LayoutData } from "./$types";
	import TextInput from "$/components/input/text-input.svelte";
	import { t } from "svelte-i18n";
	import type { UserSearchResult } from "$/gql/graphql";
	import { getContextClient } from "@urql/svelte";
	import { graphql } from "$/gql";

	const PAGE_SIZE = 24;

	export let data: LayoutData;

	let page = 1;

	let totalCount: number | null = null;
	let pageCount = 0;

	let channels: Promise<UserSearchResult> | null = null;

	$: data.streamed.emote.then((emote) => {
		channels = queryChannels(emote.id, page);
	});

	const client = getContextClient();

	async function queryChannels(emoteId: string, page: number): Promise<UserSearchResult> {
		const result = await client
			.query(
				graphql(`query EmoteChannels($emoteId: Id!, $page: Int!, $perPage: Int!) {
					emotes {
						emote(id: $emoteId) {
							channels(page: $page, perPage: $perPage) {
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
				}`),
				{ emoteId, page, perPage: PAGE_SIZE }
			)
			.toPromise();

		if (result.error || !result.data || !result.data.emotes.emote) {
			console.error(result.error);
			throw result.error;
		}

		totalCount = result.data.emotes.emote.channels.totalCount;
		pageCount = result.data.emotes.emote.channels.pageCount;

		return result.data.emotes.emote.channels as UserSearchResult;
	}
</script>

<div class="navigation">
	{#await data.streamed.emote then emote}
		<EmoteTabs id={emote.id} channelCount={totalCount} />
	{/await}
	{#if channels}
		<div class="inputs">
			<div class="buttons">
				<Button disabled={page <= 1} on:click={() => (page--)}>
					<CaretLeft slot="icon" />
				</Button>
				<Button disabled={page >= pageCount} on:click={() => (page++)}>
					<CaretRight slot="icon" />
				</Button>
			</div>
			<HideOn mobile>
				<TextInput placeholder={$t("labels.search")} style="max-width: 12.5rem">
					<MagnifyingGlass slot="icon" />
				</TextInput>
			</HideOn>
		</div>
	{/if}
</div>
<div class="channels">
	{#if channels}
		{#await channels}
			{#each Array(PAGE_SIZE) as _, i}
				<div class="preview loading-animation" style:animation-delay="{-i * 10}ms"></div>
			{/each}
		{:then result}
			{#each result.items as channel}
				<ChannelPreview user={channel} />
			{/each}
		{/await}
	{:else}
		{#each Array(PAGE_SIZE) as _, i}
			<div class="preview loading-animation" style:animation-delay="{-i * 10}ms"></div>
		{/each}
	{/if}
</div>

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;

		.inputs {
			display: flex;
			align-items: center;
			gap: 0.5rem;
		}

		.buttons {
			display: flex;
			align-items: center;
		}
	}

	.channels {
		margin-top: 1.5rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(8rem, 1fr));
		justify-content: center;
		column-gap: 0.5rem;
		row-gap: 1rem;

		.preview {
			border-radius: 0.5rem;
			background-color: var(--preview);
			height: 3rem;
		}
	}
</style>
