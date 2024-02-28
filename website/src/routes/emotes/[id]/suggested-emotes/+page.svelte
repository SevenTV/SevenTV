<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
	import type { LayoutData } from "../$types";
	import Button from "$/components/button.svelte";
	import EmoteContainer from "$/components/emote-container.svelte";

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
<EmoteContainer topMargin={1.5}>
	{#each Array(10) as _, i}
		<EmotePreview index={i} name="suggestedEmote{i}" bg="light" />
	{/each}
</EmoteContainer>

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
