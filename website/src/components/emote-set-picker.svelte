<script lang="ts">
	import { MagnifyingGlass } from "phosphor-svelte";
	import TextInput from "./input/text-input.svelte";
	import Expandable from "./expandable.svelte";
	import Checkbox from "./input/checkbox.svelte";
	import Flags from "./flags.svelte";
	import Radio from "./input/radio.svelte";

	export let radioName: string | null = null;

	$: input = radioName ? Radio : Checkbox;
</script>

<TextInput placeholder="Search Emote Set">
	<MagnifyingGlass slot="icon" />
</TextInput>
<Expandable title="Pinned">
	{#each Array(3) as _, i}
		<svelte:component this={input} option name={radioName ?? ""}>
			<div slot="left-label" class="emote-set">
				Emote Set {i + 1}
				<Flags flags={["600/1000", "default"]} />
			</div>
		</svelte:component>
	{/each}
</Expandable>
<Expandable title="ayyybubu">
	{#each Array(3) as _, i}
		<svelte:component this={input} option name={radioName ?? ""}>
			<div slot="left-label" class="emote-set">
				Emote Set {i + 1}
				<Flags flags={["600/1000", "default"]} />
			</div>
		</svelte:component>
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
