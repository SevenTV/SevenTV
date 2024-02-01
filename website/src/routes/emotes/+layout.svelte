<script lang="ts">
	import Expandable from "$/components/expandable.svelte";
	import { page } from "$app/stores";
	import {
		faArrowDownWideShort,
		faArrowUpWideShort,
		faFaceGrinWink,
		faFolder,
	} from "@fortawesome/pro-regular-svg-icons";
	import { faChevronLeft, faChevronRight, faXmark } from "@fortawesome/pro-solid-svg-icons";
	import Fa from "svelte-fa";
	import { fly } from "svelte/transition";
	import { sideBar } from "$/lib/stores";
	import Checkbox from "$/components/checkbox.svelte";
	import Select from "$/components/select.svelte";

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
		<div class="side-bar" transition:fly={{ x: -16 * 16, duration: 200, opacity: 1 }}>
			<button class="button collapse" on:click={() => ($sideBar = false)}>
				<Fa icon={faChevronLeft} fw />
			</button>
			<h1>Directory</h1>
			<div class="link-list">
				<a class="button big" href="/emotes" class:secondary={$page.url.pathname === "/emotes"}>
					<Fa icon={faFaceGrinWink} size="1.2x" fw />
					Emotes
				</a>
				<a
					class="button big"
					href="/emotes/sets"
					class:secondary={$page.url.pathname === "/emotes/sets"}
				>
					<Fa icon={faFolder} size="1.2x" fw />
					Emote Sets
				</a>
			</div>
			<hr />
			<Expandable title="Sorting">
				<div class="sorting">
					<div class="select">
						<Select
							options={[
								{ name: "Name", key: 0 },
								{ name: "Date", key: 1 },
							]}
						/>
					</div>
					<button class="button secondary" on:click={() => (sortAsc = !sortAsc)}>
						<Fa icon={sortAsc ? faArrowUpWideShort : faArrowDownWideShort} size="1.2x" />
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
								<Fa icon={faXmark} />
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
		</div>
	{:else}
		<button class="button expand" on:click={() => ($sideBar = true)}>
			<Fa icon={faChevronRight} fw />
		</button>
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
		}
	}

	.expand {
		position: absolute;
		top: 1rem;
		left: 0.5rem;
	}

	.sorting {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		.select {
			flex-grow: 1;
		}

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
