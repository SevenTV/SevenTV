<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
	import type { LayoutData } from "../$types";
	import Button from "$/components/button.svelte";

	export let data: LayoutData;
</script>

<div class="navigation">
    <EmoteTabs id={data.id} />
	<div class="buttons">
		<Button disabled>
			<CaretLeft slot="icon" />
		</Button>
		<Button>
			<CaretRight slot="icon" />
		</Button>
	</div>
</div>
<div class="emotes">
	{#each Array(10) as _, i}
		<EmotePreview index={i} name="suggestedEmote{i}" bg="light" />
	{/each}
</div>

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

	.emotes {
		margin-top: 1.5rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		place-items: center;
		gap: 1rem;
	}
</style>
