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
	} from "phosphor-svelte";
	import Button from "$/components/button.svelte";
	import TagsInput from "$/components/tags-input.svelte";
	import TabLink from "$/components/tab-link.svelte";

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
		<div class="link-list">
			<TabLink href="/emotes" title="Emotes" big matcher={menuMatcher}>
				<Smiley />
				<Smiley width="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/sets" title="Emote Sets" big matcher={menuMatcher}>
				<FolderSimple />
				<FolderSimple width="fill" slot="active" />
			</TabLink>
			<TabLink href="/emotes/bookmarked" title="Bookmarked" big matcher={menuMatcher}>
				<BookmarkSimple />
				<BookmarkSimple width="fill" slot="active" />
			</TabLink>
		</div>
		<hr />
		<Expandable title="Sorting">
			<div class="sorting">
				<Select options={["Name", "Date"]} grow />
				<Button primary on:click={() => (sortAsc = !sortAsc)}>
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
		<Expandable title="Tags">
			<TagsInput />
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
	}

	.filters {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}
</style>
