<script lang="ts">
	import StoreSection from "./store-section.svelte";
	import PaintPreview from "../paint-preview.svelte";
	import { DotsThreeVertical, PaintBrush, Repeat } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import moment from "moment/min/moment-with-locales";
	import Button from "../input/button.svelte";
	import DropDown from "../drop-down.svelte";
</script>

<StoreSection title={$t("pages.store.subscription.monthly_paints")}>
	{#snippet header()}
		<div class="buttons">
			<div class="renew-countdown">
				<Repeat />
				<span>{moment().endOf("month").fromNow(true)}</span>
			</div>
			<DropDown>
				{#snippet dropdown()}
					<Button big href="/cosmetics">
						{#snippet icon()}
							<PaintBrush />
						{/snippet}
						Your Paints
					</Button>
				{/snippet}
				<Button secondary>
					{#snippet icon()}
						<DotsThreeVertical />
					{/snippet}
				</Button>
			</DropDown>
		</div>
	{/snippet}
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
