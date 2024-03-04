<script lang="ts">
	import Expandable from "$/components/expandable.svelte";
	import Checkbox from "$/components/checkbox.svelte";
	import Select from "$/components/select.svelte";
	import {
		Smiley,
		FolderSimple,
		SortAscending,
		SortDescending,
		BookmarkSimple,
		PencilSimple,
	} from "phosphor-svelte";
	import Button from "$/components/button.svelte";
	import TagsInput from "$/components/input/tags-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import SearchBar from "$/components/input/search-bar.svelte";

	let sortAsc = false;

	function menuMatcher(id: string | null, _url: URL, href: string) {
		switch (href) {
			case "/emotes":
				return id?.startsWith("/emotes/(directory)/(emotes)") ?? false;
			case "/emotes/sets":
				return id?.startsWith("/emotes/(directory)/sets") ?? false;
			case "/emotes/bookmarked":
				return id?.startsWith("/emotes/(directory)/bookmarked") ?? false;
		}
		return false;
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>Directory</h1>
		<nav class="link-list">
			<TabLink href="/emotes" title="Emotes" big matcher={menuMatcher}>
				<Smiley />
				<Smiley weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/sets" title="Emote Sets" big matcher={menuMatcher}>
				<FolderSimple />
				<FolderSimple weight="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/bookmarked" title="Bookmarked" big matcher={menuMatcher}>
				<BookmarkSimple />
				<BookmarkSimple weight="fill" slot="active" />
			</TabLink>
		</nav>
		<hr />
		<Expandable title="Search">
			<SearchBar placeholder="Emote" grow />
		</Expandable>
		<Expandable title="Tags">
			<TagsInput />
		</Expandable>
		<Expandable title="Sorting">
			<div class="row">
				<Select options={["Alphabetical", "Upload Date"]} grow />
				<Button secondary on:click={() => (sortAsc = !sortAsc)}>
					<svelte:fragment slot="icon">
						{#if sortAsc}
							<SortAscending />
						{:else}
							<SortDescending />
						{/if}
					</svelte:fragment>
				</Button>
			</div>
		</Expandable>
		<Expandable title="Filters">
			<div class="filters">
				<Checkbox label="Animated" />
				<Checkbox label="Static" />
				<Checkbox label="Overlaying" />
				<Checkbox label="Case Sensitive" />
				<Checkbox label="Exact Match" />
			</div>
		</Expandable>
		<Expandable title="Size">
			<div class="row">
				<Select options={["Any Size", "idk what this is"]} grow />
				<Button secondary>
					<PencilSimple slot="icon" />
				</Button>
			</div>
		</Expandable>
	</aside>
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.filters {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
