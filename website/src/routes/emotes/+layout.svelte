<script lang="ts">
	import Expandable from "$/components/expandable.svelte";
	import { page } from "$app/stores";
	import { fly } from "svelte/transition";
	import { sideBar } from "$/lib/stores";
	import Checkbox from "$/components/checkbox.svelte";
	import Select from "$/components/select.svelte";
	import {
		ArrowLineLeft,
		Smiley,
		FolderSimple,
		SortAscending,
		SortDescending,
		X,
	} from "phosphor-svelte";

	let sortAsc = false;

	let tags = ["lorem", "ipsum"];

	function removeTag(i: number) {
		tags.splice(i, 1);
		tags = [...tags];
	}

	let tagInput: string;

	function onTagInput(e: KeyboardEvent) {
		if (e.key === "Enter" && tagInput) {
			tags = [...tags, tagInput];
			tagInput = "";
			e.preventDefault();
		}
	}
</script>

<div class="side-bar-layout">
	{#if $sideBar}
		<aside class="side-bar" transition:fly={{ x: -16 * 16, duration: 200, opacity: 1 }}>
			<button class="button square collapse" on:click={() => ($sideBar = false)}>
				<ArrowLineLeft />
			</button>
			<h1>Directory</h1>
			<div class="link-list">
				<a
					class="button big"
					href="/emotes"
					class:secondary={$page.route.id?.startsWith("/emotes/(emotes)")}
				>
					<Smiley />
					Emotes
				</a>
				<a
					class="button big"
					href="/emotes/sets"
					class:secondary={$page.route.id?.startsWith("/emotes/sets")}
				>
					<FolderSimple />
					Emote Sets
				</a>
			</div>
			<hr />
			<Expandable title="Sorting">
				<div class="sorting">
					<Select options={["Name", "Date"]} grow />
					<button class="button square secondary" on:click={() => (sortAsc = !sortAsc)}>
						{#if sortAsc}
							<SortAscending />
						{:else}
							<SortDescending />
						{/if}
					</button>
				</div>
			</Expandable>
			<Expandable title="Tags">
				<input type="text" placeholder="Add tags" bind:value={tagInput} on:keypress={onTagInput} />
				{#if tags && tags.length > 0}
					<div class="tags">
						{#each tags as tag, i}
							<button class="button secondary tag" on:click={() => removeTag(i)}>
								<span>{tag}</span>
								<X />
							</button>
						{/each}
					</div>
				{/if}
			</Expandable>
			<Expandable title="Filters">
				<div class="filters">
					<Checkbox label="Zero-Width" />
					<Checkbox label="Animated" />
					<Checkbox label="Exact Match" />
					<Checkbox label="Case Sensitive" />
					<Checkbox label="Ignore Tags" />
					<Checkbox label="Personal Use" />
				</div>
			</Expandable>
			<Expandable title="Ratio">Ratio</Expandable>
		</aside>
	{/if}
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.side-bar-layout {
		position: relative;
	}

	.side-bar {
		z-index: 1;

		h1 {
			font-size: 1.125rem;
			font-weight: 600;
			margin: 0.25rem 0;
		}

		.button.collapse {
			position: absolute;
			top: 1rem;
			right: 1rem;
		}

		.link-list {
			display: flex;
			flex-direction: column;
			gap: 0.25rem;
		}
	}

	.sorting {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		& > .button {
			padding: 0.5rem;
		}
	}

	.tags {
		margin-top: 0.75rem;

		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.tag {
		padding: 0.4rem 0.75rem 0.4rem 1rem;
		font-weight: 500;
		max-width: 100%;

		& > span {
			overflow: hidden;
			text-overflow: ellipsis;
		}
	}

	.filters {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}
</style>
