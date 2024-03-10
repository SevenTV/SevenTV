<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import PaintPreview from "../paint-preview.svelte";
	import { DotsThreeVertical, Repeat } from "phosphor-svelte";
	import Button from "../input/button.svelte";
</script>

<StoreSection title="Monthly Paints">
	<div class="buttons" slot="header">
		<div class="renew-countdown">
			<Repeat />
			<span>12 d</span>
		</div>
		<Button secondary>
			<DotsThreeVertical slot="icon" />
		</Button>
	</div>
	{#each Array(2) as _}
		<PaintPreview />
	{/each}
</StoreSection>

<style lang="scss">
	.buttons {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.renew-countdown {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		white-space: nowrap;

		color: var(--text-light);
		font-size: 0.75rem;
	}
</style>
