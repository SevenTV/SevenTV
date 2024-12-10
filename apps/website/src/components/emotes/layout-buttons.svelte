<script lang="ts">
	import { type Layout } from "$/lib/layout";
	import { GridFour, GridNine, List } from "phosphor-svelte";
	import Button from "../input/button.svelte";

	interface Props {
		value: Layout;
		allowedLayouts?: Layout[];
	}

	let { value = $bindable(), allowedLayouts = ["big-grid", "small-grid"] }: Props = $props();
</script>

{#each allowedLayouts as layout}
	<Button secondary={value === layout} onclick={() => (value = layout)}>
		{#snippet icon()}
			{#if layout === "big-grid"}
				<GridFour />
			{:else if layout === "small-grid"}
				<GridNine />
			{:else if layout === "list"}
				<List />
			{/if}
		{/snippet}
	</Button>
{/each}
