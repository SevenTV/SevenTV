<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import { NotePencil } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import { t } from "svelte-i18n";
	import EmoteSetPreview from "../emote-set-preview.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { user } from "$/lib/auth";
	import Spinner from "../spinner.svelte";
	import type { EmoteSet } from "$/gql/graphql";

	async function loadPersonalEmoteSet(userId: string) {
		const res = await gqlClient()
			.query(
				graphql(`
					query PersonalEmoteSet($userId: Id!) {
						users {
							user(id: $userId) {
								personalEmoteSet {
									id
									name
									capacity
									kind
									emotes(page: 1, perPage: 12) {
										items {
											emote {
												images {
													url
													mime
													size
													scale
													width
													frameCount
												}
											}
										}
										totalCount
									}
								}
							}
						}
					}
				`),
				{ userId },
			)
			.toPromise();

		return res.data?.users.user?.personalEmoteSet as EmoteSet;
	}

	let personalEmoteSet = $derived($user ? loadPersonalEmoteSet($user.id) : undefined);
</script>

<StoreSection title={$t("common.personal_emotes")}>
	{#await personalEmoteSet}
		<div class="container">
			<Spinner />
		</div>
	{:then emoteSet}
		{#if emoteSet}
			<div class="container">
				<EmoteSetPreview data={emoteSet} bg="light" />
			</div>
			<Button secondary style="align-self: flex-end" href="/emote-sets/{emoteSet.id}">
				{#snippet icon()}
					<NotePencil />
				{/snippet}
				{$t("labels.edit")}
			</Button>
		{:else}
			<div class="container">No Personal Set</div>
		{/if}
	{/await}
</StoreSection>

<style lang="scss">
	.container {
		flex-grow: 1;

		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
