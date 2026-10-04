<script lang="ts">
	import type { Emote, EmoteSearchResult, User } from "$/gql/graphql";
	import { onMount, type ComponentProps, type Snippet } from "svelte";
	import TextInput from "./input/text-input.svelte";
	import { gql } from "@urql/svelte";
	import { gqlClient } from "$/lib/gql";
	import Button from "./input/button.svelte";
	import ResponsiveImage from "./responsive-image.svelte";
	import Spinner from "./spinner.svelte";
	import { Smiley } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	type Props = {
		userID: string;
		selected: Emote | undefined;
		children?: Snippet;
	} & ComponentProps<typeof TextInput>;

	let { userID, selected = $bindable(), children, ...restProps }: Props = $props();

	let userEmotes = $state<EmoteSearchResult>();
	let searchInput = $state<string>("");

	let results = $derived(userEmotes ? searchFilter() : null);

	function searchFilter() {
		if (!userEmotes || !userEmotes["items"]) return null;
		if (!searchInput.length) return userEmotes["items"];

		return userEmotes["items"].filter((emote) =>
			emote.defaultName.toLowerCase().includes(searchInput.toLowerCase()),
		);
	}

	async function load(): Promise<EmoteSearchResult> {
		const variables = {
			id: userID,
		};

		const gql_query = gql`
			query UserOwnedEmotes($id: Id!) {
				users {
					user(id: $id) {
						ownedEmotes {
							id
							defaultName
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
				}
			}
		`;

		const res = await gqlClient().query(gql_query, variables).toPromise();
		if (res.error || !res.data) {
			throw res.error;
		}
		const emotes = res.data.users.user?.ownedEmotes;
		if (!emotes) {
			throw new Error("No emotes found");
		}
		return {
			items: emotes as Emote[],
			totalCount: emotes.length,
			pageCount: 1,
		};
	}

	const bindEmote = (e: Emote) => {
		selected = e;
		searchInput = e.defaultName;
	};

	const clearValue = () => (selected = undefined);

	onMount(async () => (userEmotes = await load()));
</script>

<TextInput
	type="text"
	{...restProps}
	placeholder={$t("labels.search_emotes", { values: { count: 1 } })}
	bind:value={searchInput}
	oninput={clearValue}
>
	{#snippet icon()}
		{#if !selected}
			{#await userEmotes}
				<Spinner />
			{:then _}
				<Smiley />
			{/await}
		{:else}
			<!-- wrapped to prevent weird display -->
			<span><ResponsiveImage images={selected.images} width={10 * 2} /></span>
		{/if}
	{/snippet}
	{@render children?.()}
	{#if results && results.length}
		<div class="popup-results results">
			{#each results as result}
				<Button class="item" onclick={() => bindEmote(result)}>
					{#snippet icon()}
						<ResponsiveImage images={result.images} width={13 * 2} />
					{/snippet}
					{result.defaultName}
				</Button>
			{/each}
		</div>
	{/if}
</TextInput>

<style lang="scss">
	:global(label.input:has(input:enabled)):focus-within > .popup-results {
		display: flex;
	}

	span {
		:global {
			.image {
				vertical-align: middle;
			}
		}
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
		overflow-x: hidden;

		flex-direction: column;

		& > :global(.button) {
			animation: expand-down 0.2s forwards;
		}
	}

	.results {
		max-height: 10rem;
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
