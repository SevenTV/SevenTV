<script lang="ts">
	import { emotesLayout } from "$/lib/layout";
	import { GridFour, GridNine } from "phosphor-svelte";
	import Button from "../input/button.svelte";
</script>

<Button secondary={$emotesLayout === "big-grid"} onclick={() => ($emotesLayout = "big-grid")}>
	{#snippet icon()}
		<GridFour />
	{/snippet}
</Button>
<Button secondary={$emotesLayout === "small-grid"} onclick={() => ($emotesLayout = "small-grid")}>
	{#snippet icon()}
		<GridNine />
	{/snippet}
</Button>
