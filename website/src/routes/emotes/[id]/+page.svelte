<script lang="ts">
	import Button from "$/components/button.svelte";
	import ChannelPreview from "$/components/channel-preview.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import SearchBar from "$/components/search-bar.svelte";
	import { CaretLeft, CaretRight } from "phosphor-svelte";
</script>

<div class="navigation">
	<div class="buttons">
		<Button disabled style="color: var(--text-light)">
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
<div class="channels">
	{#each Array(20) as _, i}
		<ChannelPreview index={i} user="channel{i}" bg="light" />
	{/each}
</div>

<style lang="scss">
	.navigation {
		position: absolute;
		top: 1rem;
		right: 1rem;

		display: flex;
		align-items: center;
		gap: 0.5rem;

		.buttons {
			display: flex;
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
