<script lang="ts">
	import { MagnifyingGlass } from "phosphor-svelte";
	import TextInput from "./input/text-input.svelte";
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import Radio from "./input/radio.svelte";
	import { t } from "svelte-i18n";

	let { radioName }: { radioName?: string } = $props();
</script>

{#snippet pickerLeftLabel()}
	<div class="emote-set">
		Emote Set
		<Flags flags={["600/1000", "default"]} />
	</div>
{/snippet}

<TextInput placeholder={$t("labels.search_emote_set")}>
	{#snippet icon()}
		<MagnifyingGlass />
	{/snippet}
</TextInput>
<Expandable title={$t("common.pinned")}>
	{#each Array(3) as _}
		{#if radioName}
			<Radio name={radioName} option leftLabel={pickerLeftLabel} />
		{:else}
			<Checkbox option leftLabel={pickerLeftLabel} />
		{/if}
	{/each}
</Expandable>
<Expandable title="ayyybubu">
	{#each Array(3) as _}
		{#if radioName}
			<Radio name={radioName} option leftLabel={pickerLeftLabel} />
		{:else}
			<Checkbox option leftLabel={pickerLeftLabel} />
		{/if}
	{/each}
</Expandable>

<style lang="scss">
	.emote-set {
		font-size: 0.875rem;
		font-weight: 500;

		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
