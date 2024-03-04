<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import Select from "$/components/select.svelte";
	import type { LayoutData } from "../$types";

	export let data: LayoutData;
</script>

<div class="navigation">
	<EmoteTabs id={data.id} />
	<div class="buttons">
		<Select options={["This Week", "This Month", "This Century"]} />
		<Select options={["Channels"]} />
	</div>
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
		gap: 0.5rem;
	}
</style>
