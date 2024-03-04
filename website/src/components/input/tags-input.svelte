<script lang="ts">
	import { Tag, X } from "phosphor-svelte";
	import Button from "../button.svelte";
	import TextInput from "./text-input.svelte";

	export let tags = ["lorem", "ipsum"];

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

<TextInput placeholder="Enter tags" bind:value={tagInput} on:keypress={onTagInput}>
	<Tag />
</TextInput>
{#if tags && tags.length > 0}
	<div class="tags">
		{#each tags as tag, i}
			<Button secondary on:click={() => removeTag(i)}>
				<span>{tag}</span>
				<X size="1rem" />
			</Button>
		{/each}
	</div>
{/if}

<style lang="scss">
	.tags {
		margin-top: 0.75rem;

		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex-wrap: wrap;

		& > :global(.button) {
			font-size: 0.875rem;
			padding: 0.22rem 0.5rem 0.22rem 0.62rem;
			font-weight: 500;
			max-width: 100%;
		}

		& > :global(.button > span) {
			overflow: hidden;
			text-overflow: ellipsis;
		}
	}
</style>
