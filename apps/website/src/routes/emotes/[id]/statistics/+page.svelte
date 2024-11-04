<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import Select from "$/components/input/select.svelte";
	import { t } from "svelte-i18n";
	import type { PageData } from "./$types";

	let { data }: { data: PageData } = $props();
</script>

<div class="navigation">
	{#await data.streamed.emote then emote}
		<EmoteTabs id={emote.id} />
	{/await}
	<div class="buttons">
		<Select
			options={[
				{ value: "week", label: $t("labels.time_select.week") },
				{ value: "month", label: $t("labels.time_select.month") },
				{ value: "year", label: $t("labels.time_select.year") },
			]}
		/>
		<Select
			options={[{ value: "channels", label: $t("common.channels", { values: { count: 2 } }) }]}
		/>
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
