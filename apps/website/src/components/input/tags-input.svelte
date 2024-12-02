<script lang="ts">
	import { Tag, X } from "phosphor-svelte";
	import Button from "./button.svelte";
	import TextInput from "./text-input.svelte";
	import { t } from "svelte-i18n";
	import type { Snippet } from "svelte";

	const LIMIT = 10;

	interface Props {
		tags?: string[];
		children?: Snippet;
	}

	let { tags = $bindable(["lorem", "ipsum"]), children }: Props = $props();

	function removeTag(i: number) {
		tags.splice(i, 1);
		tags = [...tags];
	}

	let tagInput: string | undefined = $state();

	function onTagInput(e: KeyboardEvent) {
		if (e.key === "Enter" && tagInput && tags.length < LIMIT) {
			tags = [...tags, tagInput];
			tagInput = "";
			e.preventDefault();
		}
	}
</script>

<TextInput placeholder={$t("labels.enter_tags")} bind:value={tagInput} onkeypress={onTagInput}>
	{@render children?.()}
	{#snippet icon()}
		<Tag />
	{/snippet}
</TextInput>
{#if tags && tags.length > 0}
	<div class="tags">
		{#each tags as tag, i}
			<Button secondary onclick={() => removeTag(i)}>
				<span>{tag}</span>
				<X size={1 * 16} />
			</Button>
		{/each}
	</div>
{/if}

<style lang="scss">
	.tags {
		margin-top: 0.25rem;

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
