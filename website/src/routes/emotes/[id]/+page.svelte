<script lang="ts">
	import Button from "$/components/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import SearchBar from "$/components/search-bar.svelte";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
	import type { LayoutData } from "./$types";

	export let data: LayoutData;
</script>

<div class="navigation">
	<EmoteTabs id={data.id} />
	<div class="inputs">
		<div class="buttons">
			<Button disabled>
				<CaretLeft slot="icon" />
			</Button>
			<Button>
				<CaretRight slot="icon" />
			</Button>
		</div>
		<HideOn mobile>
			<SearchBar />
		</HideOn>
	</div>
</div>
<div class="channels">
	{#each Array(20) as _, i}
		<ChannelPreview index={i} user="channel{i}" bg="light" />
	{/each}
</div>

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;

		.inputs {
			display: flex;
			align-items: center;
			gap: 0.5rem;
		}

		.buttons {
			display: flex;
			align-items: center;
		}
	}

	.channels {
		margin-top: 1.5rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
		justify-content: center;
		gap: 1rem;
	}
</style>
