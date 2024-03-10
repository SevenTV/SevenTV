<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import Select from "$/components/input/select.svelte";
	import type { LayoutData } from "../$types";

	export let data: LayoutData;
</script>

<div class="navigation">
	<EmoteTabs id={data.id} />
	<div class="buttons">
		<Select
			options={[
				{ value: "week", label: "This Week" },
				{ value: "month", label: "This Month" },
				{ value: "century", label: "This Century" },
			]}
		/>
		<Select options={[{ value: "channels", label: "Channels" }]} />
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
