<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
	import Button from "$/components/input/button.svelte";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();
</script>

<div class="navigation">
	{#await data.streamed.emote then emote}
		<EmoteTabs id={emote.id} />
	{/await}
	<div class="buttons">
		<Button disabled>
			{#snippet icon()}
				<CaretLeft />
			{/snippet}
		</Button>
		<Button>
			{#snippet icon()}
				<CaretRight />
			{/snippet}
		</Button>
	</div>
</div>

<!-- <EmoteContainer style="margin-top: 1.5rem">
	{#each Array(10) as _, i}
		<EmotePreview index={i} name="suggestedEmote{i}" bg="light" />
	{/each}
</EmoteContainer> -->

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.buttons {
		display: flex;
		align-items: center;
	}
</style>
