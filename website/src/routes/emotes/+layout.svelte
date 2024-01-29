<script lang="ts">
	import Expandable from "$/components/expandable.svelte";
	import { page } from "$app/stores";
	import { faFaceGrinWink, faFolder } from "@fortawesome/pro-regular-svg-icons";
	import { faChevronLeft, faChevronRight } from "@fortawesome/pro-solid-svg-icons";
	import Fa from "svelte-fa";
	import { fly } from "svelte/transition";
	import { sideBar } from "$/lib/stores";
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
			<Expandable title="Sorting">Sort</Expandable>
			<Expandable title="Tags">Tags</Expandable>
			<Expandable title="Filters">Filters</Expandable>
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
		position: relative;

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
</style>
