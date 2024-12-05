<script lang="ts">
	import { User as UserIcon } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import { t } from "svelte-i18n";
	import type { Emote, User } from "$/gql/graphql";
	import Spinner from "../spinner.svelte";
	import ChannelPreview from "../channel-preview.svelte";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { updateOwner } from "$/lib/emoteMutations";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data = $bindable() }: Props = $props();

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

	$effect(() => {
		results.then(() => {
			input.focus();
		});
	});

	// svelte-ignore non_reactive_update
	let input: ReturnType<typeof TextInput>;

	let loading = $state(false);

	async function submit() {
		if (!recipient) {
			return;
		}

		loading = true;

		const newData = await updateOwner(data.id, recipient.id);

		if (newData) {
			data = newData;
		}

		loading = false;
		mode = "hidden";
	}
</script>

<EmoteDialog
	width={35}
	title={$t("dialogs.transfer_emote.title", { values: { emote: data.defaultName } })}
	bind:mode
	{data}
>
	<span class="details">
		{$t("dialogs.transfer_emote.details", { values: { emote: data.defaultName } })}
	</span>
	{#if recipient}
		<span class="label">{$t("dialogs.transfer_emote.receipient")}</span>
		<ChannelPreview
			user={recipient}
			size={2}
			onclick={(e) => {
				e.preventDefault();
				recipient = undefined;
			}}
		/>
	{:else}
		<TextInput
			type="text"
			placeholder={$t("labels.search_users", { values: { count: 1 } })}
			bind:this={input}
			bind:value={query}
		>
			{#snippet icon()}
				{#await results}
					<Spinner />
				{:then _}
					<UserIcon />
				{/await}
			{/snippet}
			<span class="label">{$t("dialogs.transfer_emote.receipient")}</span>
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
	{#snippet buttons()}
		<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		<Button
			primary
			submit
			disabled={loading}
			onclick={submit}
			icon={loading ? loadingSpinner : undefined}
		>
			{$t("dialogs.transfer_emote.transfer")}
		</Button>
	{/snippet}
</EmoteDialog>

<style lang="scss">
	.details {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
