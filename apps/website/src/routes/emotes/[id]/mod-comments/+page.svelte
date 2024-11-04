<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();
</script>

<div class="navigation">
	{#await data.streamed.emote then emote}
		<EmoteTabs id={emote.id} />
	{/await}
</div>

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}
</style>
