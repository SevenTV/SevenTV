<script lang="ts">
	import TextInput from "../input/text-input.svelte";
	import { type DialogMode } from "./dialog.svelte";
	import Button from "../input/button.svelte";
	import EmoteDialog from "./emote-dialog.svelte";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import { t } from "svelte-i18n";
	import { untrack, type Snippet } from "svelte";
	import type { Emote } from "$/gql/graphql";
	import { gqlClient } from "$/lib/gql";
	import { graphql } from "$/gql";
	import { user } from "$/lib/auth";
	import Spinner from "../spinner.svelte";

	interface Props {
		mode: DialogMode;
		data: Emote;
	}

	let { mode = $bindable("hidden"), data }: Props = $props();

	let originalState: { [key: string]: boolean }; // not reactive
	let pickedEmoteSets: { [key: string]: boolean } = $state({});

	let alias = $state(data.defaultName);

	async function queryInSet(emoteId: string, setIds: string[]) {
		const res = await gqlClient().query(
			graphql(`
				query IsInSet($id: Id!, $setIds: [Id!]!) {
					emotes {
						emote(id: $id) {
							inEmoteSets(emoteSetIds: $setIds) {
								emoteSetId
								emote {
									id
									alias
									flags {
										zeroWidth
									}
								}
							}
						}
					}
				}
			`),
			{ id: emoteId, setIds },
		);

		return res.data?.emotes.emote?.inEmoteSets;
	}

	let inSet = $derived(
		$user
			? queryInSet(
					data.id,
					$user.editableEmoteSets.map((s) => s.id),
				)
			: undefined,
	);

	$effect(() => {
		if ($user) {
			untrack(() => (pickedEmoteSets = {}));

			alias; // Make alias a dependency

			inSet?.then((inSets) => {
				if (!inSets) {
					return;
				}

				for (const inSet of inSets) {
					pickedEmoteSets[inSet.emoteSetId] = inSet.emote ? inSet.emote.alias === alias : false;
				}

				originalState = { ...pickedEmoteSets };
			});
		}
	});

	let toAdd = $derived(
		Object.keys(pickedEmoteSets).filter((k) => pickedEmoteSets[k] && !originalState[k]),
	);

	let submitting = $state(false);

	async function submit() {
		submitting = true;

		for (const setId of toAdd) {
			await gqlClient()
				.mutation(
					graphql(`
						mutation AddEmoteToSet($setId: Id!, $emote: EmoteSetEmoteId!) {
							emoteSet(id: $setId) {
								addEmote(emote: { id: $emote }) {
									id
								}
							}
						}
					`),
					{ setId, emote: { emoteId: data.id, alias } },
				)
				.toPromise();

			mode = "hidden";
		}
	}
</script>

{#snippet buttons()}
	<Button onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
	{#if submitting}
		<Button primary submit disabled>
			{#snippet iconRight()}
				<Spinner />
			{/snippet}
			{$t("labels.confirm")}
		</Button>
	{:else}
		<Button primary submit onclick={submit} disabled={toAdd.length === 0}>
			{$t("labels.confirm")}
		</Button>
	{/if}
{/snippet}

<EmoteDialog
	title={$t("dialogs.add_emote.title", { values: { emote: data.defaultName } })}
	bind:mode
	{buttons}
	{data}
>
	{#snippet preview()}
		<TextInput
			placeholder={$t("labels.emote_name")}
			style="max-width: 12.5rem"
			disabled={submitting}
			bind:value={alias}
		>
			<span class="label">{$t("dialogs.add_emote.change_name")}</span>
		</TextInput>
	{/snippet}
	<EmoteSetPicker disabled={submitting} bind:value={pickedEmoteSets} />
</EmoteDialog>

<style lang="scss">
	.label {
		font-size: 0.875rem;
		font-weight: 500;
	}
</style>
