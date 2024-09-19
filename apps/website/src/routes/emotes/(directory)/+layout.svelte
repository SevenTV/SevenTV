<script lang="ts">
	import Expandable from "$/components/expandable.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import { Smiley, FolderSimple, BookmarkSimple, MagnifyingGlass } from "phosphor-svelte";
	import TagsInput from "$/components/input/tags-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { t } from "svelte-i18n";
	import { user } from "$/store/auth";
	import { page } from "$app/stores";
	import { goto } from "$app/navigation";

	const TAGS_SEPERATOR = " ";

	let query: string | null = $page.url.searchParams.get("q");
	let numPage: string | null = $page.url.searchParams.get("p");
	let tags: string[] = $page.url.searchParams.get("t")?.split(TAGS_SEPERATOR) ?? [];
	let animated: boolean = $page.url.searchParams.get("a") == "1";
	let staticFilter: boolean = $page.url.searchParams.get("s") == "1";
	let overlaying: boolean = $page.url.searchParams.get("o") == "1";
	let exactMatch: boolean = $page.url.searchParams.get("em") == "1";

	$: {
		let url = new URL($page.url);

		if (query) {
			url.searchParams.set("q", query);
		} else {
			url.searchParams.delete("q");
		}

		if (numPage) {
			url.searchParams.set("p", numPage);
		} else {
			url.searchParams.delete("p");
		}

		if (tags) {
			url.searchParams.set("t", tags.join(TAGS_SEPERATOR));
		} else {
			url.searchParams.delete("t");
		}

		goto(url, { replaceState: true, noScroll: true, keepFocus: true });
	}

	function menuMatcher(id: string | null, _url: URL, href: string | null) {
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
				<Smiley weight="fill" slot="active" />
			</TabLink>
			<TabLink
				href="/emotes/sets"
				title={$t("common.emote_sets", { values: { count: 2 } })}
				big
				matcher={menuMatcher}
			>
				<FolderSimple />
				<FolderSimple weight="fill" slot="active" />
			</TabLink>
			{#if $user}
				<TabLink
					href="/emotes/bookmarked"
					title={$t("pages.directory.bookmarked")}
					big
					matcher={menuMatcher}
				>
					<BookmarkSimple />
					<BookmarkSimple weight="fill" slot="active" />
				</TabLink>
			{/if}
		</nav>
		<hr />
		<Expandable title={$t("labels.search")}>
			<TextInput placeholder={$t("common.emotes", { values: { count: 1 } })} bind:value={query}>
				<MagnifyingGlass slot="icon" />
			</TextInput>
		</Expandable>
		<Expandable title={$t("labels.tags")} expanded={false}>
			<TagsInput bind:tags={tags} />
		</Expandable>
		<!-- <Expandable title={$t("pages.directory.sorting.title")} expanded={false}>
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
		<Expandable title={$t("labels.filters")} expanded={false}>
			<div class="filters">
				<Checkbox bind:value={animated}>{$t("pages.directory.filters.animated")}</Checkbox>
				<Checkbox bind:value={staticFilter}>{$t("pages.directory.filters.static")}</Checkbox>
				<Checkbox bind:value={overlaying}>{$t("flags.overlaying")}</Checkbox>
				<Checkbox bind:value={exactMatch}>{$t("pages.directory.filters.exact_match")}</Checkbox>
			</div>
		</Expandable>
		<!-- <Expandable title={$t("pages.directory.size.title")} expanded={false}>
			<div class="row">
				<Select
					options={[
						{ value: "any", label: $t("pages.directory.size.any") },
						{ value: "", label: "idk what this is" },
					]}
					grow
				/>
				<Button secondary>
					<PencilSimple slot="icon" />
				</Button>
			</div>
		</Expandable> -->
	</aside>
	<div class="content">
		<slot />
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
