<script lang="ts" module>
	export const TAGS_SEPERATOR = " ";
</script>

<script lang="ts">
	import Expandable from "$/components/expandable.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import { Smiley, FolderSimple, BookmarkSimple, MagnifyingGlass } from "phosphor-svelte";
	import TagsInput from "$/components/input/tags-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { t } from "svelte-i18n";
	import { user } from "$/lib/auth";
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";
	import type { Snippet } from "svelte";

	let { children }: { children: Snippet } = $props();

	let query: string | undefined = $state($page.url.searchParams.get("q") ?? undefined);
	let tags: string[] = $state($page.url.searchParams.get("t")?.split(TAGS_SEPERATOR) ?? []);
	let animated: boolean = $state($page.url.searchParams.get("a") == "1");
	let staticFilter: boolean = $state($page.url.searchParams.get("s") == "1");
	let overlaying: boolean = $state($page.url.searchParams.get("o") == "1");
	let exactMatch: boolean = $state($page.url.searchParams.get("e") == "1");

	function unsetStaticFilter() {
		staticFilter = false;
	}

	function unsetAnimated() {
		animated = false;
	}

	$effect(() => {
		if (animated) {
			unsetStaticFilter();
		}
	});

	$effect(() => {
		if (staticFilter) {
			unsetAnimated();
		}
	});

	$effect(() => {
		let url = new URL($page.url);

		if (query) {
			url.searchParams.set("q", query);
		} else {
			url.searchParams.delete("q");
		}

		if (tags && tags.length > 0) {
			url.searchParams.set("t", tags.join(TAGS_SEPERATOR));
		} else {
			url.searchParams.delete("t");
		}

		if (animated) {
			url.searchParams.set("a", "1");
		} else {
			url.searchParams.delete("a");
		}

		if (staticFilter) {
			url.searchParams.set("s", "1");
		} else {
			url.searchParams.delete("s");
		}

		if (overlaying) {
			url.searchParams.set("o", "1");
		} else {
			url.searchParams.delete("o");
		}

		if (exactMatch) {
			url.searchParams.set("e", "1");
		} else {
			url.searchParams.delete("e");
		}

		goto(url, { replaceState: true, noScroll: true, keepFocus: true });
	});

	function menuMatcher(id: string | null, _url: URL, href: string | undefined) {
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
		<h1>{$t("pages.directory.title")}</h1>
		<nav class="link-list">
			<TabLink
				href="/emotes"
				title={$t("common.emotes", { values: { count: 2 } })}
				big
				matcher={menuMatcher}
			>
				<Smiley />
				{#snippet active()}
					<Smiley weight="fill" />
				{/snippet}
			</TabLink>
			<!-- <TabLink
				href="/emotes/sets"
				title={$t("common.emote_sets", { values: { count: 2 } })}
				big
				matcher={menuMatcher}
			>
				<FolderSimple />
				{#snippet active()}
					<FolderSimple weight="fill" />
				{/snippet}
			</TabLink> -->
			<!-- {#if $user}
				<TabLink
					href="/emotes/bookmarked"
					title={$t("pages.directory.bookmarked")}
					big
					matcher={menuMatcher}
				>
					<BookmarkSimple />
					{#snippet active()}
						<BookmarkSimple weight="fill" />
					{/snippet}
				</TabLink>
			{/if} -->
		</nav>
		<hr />
		<Expandable title={$t("labels.search")} expanded={true}>
			<TextInput placeholder={$t("common.emotes", { values: { count: 1 } })} bind:value={query}>
				{#snippet icon()}
					<MagnifyingGlass />
				{/snippet}
			</TextInput>
		</Expandable>
		<Expandable title={$t("labels.tags")}>
			<TagsInput bind:tags />
		</Expandable>
		<!-- <Expandable title={$t("pages.directory.sorting.title")}>
			<div class="row">
				<Select
					options={[
						{ value: "alpha", label: $t("pages.directory.sorting.alphabetical") },
						{ value: "date", label: $t("pages.directory.sorting.upload_date") },
					]}
					grow
				/>
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
		</Expandable> -->
		<Expandable title={$t("labels.filters")}>
			<div class="filters">
				<Checkbox bind:value={animated}>{$t("pages.directory.filters.animated")}</Checkbox>
				<Checkbox bind:value={staticFilter}>{$t("pages.directory.filters.static")}</Checkbox>
				<Checkbox bind:value={overlaying}>{$t("flags.overlaying")}</Checkbox>
				<Checkbox bind:value={exactMatch} disabled={!query}
					>{$t("pages.directory.filters.exact_match")}</Checkbox
				>
			</div>
		</Expandable>
		<!-- <Expandable title={$t("pages.directory.size.title")}>
			<div class="row">
				<Select
					options={[
						{ value: "any", label: $t("pages.directory.size.any") },
						{ value: "", label: "idk what this is" },
					]}
					grow
				/>
				<Button secondary>
					<PencilSimple />
				</Button>
			</div>
		</Expandable> -->
	</aside>
	<div class="content">
		{@render children()}
	</div>
</div>

<style lang="scss">
	// .row {
	// 	display: flex;
	// 	align-items: center;
	// 	gap: 0.5rem;
	// }

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
