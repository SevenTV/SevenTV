<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import type { LayoutData } from "../$types";

	export let data: LayoutData;
</script>

<div class="navigation">
	<EmoteTabs id={data.id} />
</div>

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}
</style>
