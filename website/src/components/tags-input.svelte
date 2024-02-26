<script lang="ts">
	import { X } from "phosphor-svelte";
	import Button from "./button.svelte";

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

<input type="text" placeholder="Add tags" bind:value={tagInput} on:keypress={onTagInput} />
{#if tags && tags.length > 0}
	<div class="tags">
		{#each tags as tag, i}
			<Button primary on:click={() => removeTag(i)}>
				<span>{tag}</span>
				<X slot="icon-right" size="1rem" />
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
			padding: 0.4rem 0.75rem 0.4rem 1rem;
			font-weight: 500;
			max-width: 100%;
		}

		& > :global(.button > span) {
			overflow: hidden;
			text-overflow: ellipsis;
		}
	}
</style>
