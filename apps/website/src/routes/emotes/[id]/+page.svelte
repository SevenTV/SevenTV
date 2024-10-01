<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import { CaretLeft, CaretRight, MagnifyingGlass } from "phosphor-svelte";
	import type { LayoutData } from "./$types";
	import TextInput from "$/components/input/text-input.svelte";
	import { t } from "svelte-i18n";
	import type { User } from "$/gql/graphql";
	import { getContextClient } from "@urql/svelte";
	import { graphql } from "$/gql";

	export let data: LayoutData;

	let page = 1;

	$: channels = queryChannels(data.emote.id, page);

	async function queryChannels(emoteId: string, page: number): Promise<User[]> {
		const client = getContextClient();

		const result = await client
			.query(
				graphql(`query EmoteChannels($emoteId: Id!, $page: Int!) {
					emotes {
						emote(id: $emoteId) {
							channels(page: $page, limit: 24) {
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
							}
						}
					}
				}`),
				{ emoteId, page }
			)
			.toPromise();

		if (result.error || !result.data || !result.data.emotes.emote) {
			console.error(result.error);
			throw result.error;
		}

		return result.data.emotes.emote.channels as User[];
	}
</script>

<div class="navigation">
	<EmoteTabs id={data.emote.id} />
	<div class="inputs">
		<div class="buttons">
			<Button disabled={page <= 1} on:click={() => (page--)}>
				<CaretLeft slot="icon" />
			</Button>
			<Button on:click={() => (page++)}>
				<CaretRight slot="icon" />
			</Button>
		</div>
		<HideOn mobile>
			<TextInput placeholder={$t("labels.search")} style="max-width: 12.5rem">
				<MagnifyingGlass slot="icon" />
			</TextInput>
		</HideOn>
	</div>
</div>
<div class="channels">
	{#await channels}
		Loading
	{:then result}
		{#each result as channel}
			<ChannelPreview user={channel} />
		{/each}
	{/await}
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
		gap: 1rem;
	}
</style>
