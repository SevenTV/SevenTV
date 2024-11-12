<script lang="ts">
	import { Gear } from "phosphor-svelte";
	import DropDown from "../drop-down.svelte";
	import Button from "../input/button.svelte";
	import Select from "../input/select.svelte";
	import Toggle from "../input/toggle.svelte";
	import type { ButtonOptions } from "./emote-ticket.svelte";

	let {
		actionsPosition = $bindable(),
		buttonOptions = $bindable(),
	}: { actionsPosition?: "left" | "right"; buttonOptions: ButtonOptions } = $props();
</script>

<DropDown align={actionsPosition}>
	<Button>
		{#snippet icon()}
			<Gear />
		{/snippet}
	</Button>
	{#snippet dropdown()}
		<div class="dropdown">
			{#if actionsPosition}
				<Select
					bind:selected={actionsPosition}
					options={[
						{ value: "left", label: "Left" },
						{ value: "right", label: "Right" },
					]}
					grow
				/>
			{/if}
			<Toggle bind:value={buttonOptions.merge}>Show Merge</Toggle>
			<Toggle bind:value={buttonOptions.delete}>Show Delete</Toggle>
			<Toggle bind:value={buttonOptions.unlist}>Show Unlist</Toggle>
			<Toggle bind:value={buttonOptions.approve}>Show Approve</Toggle>
		</div>
	{/snippet}
</DropDown>

<style lang="scss">
	.dropdown {
		padding: 1rem;
		min-width: 11rem;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;

		color: var(--text);
		font-size: 0.875rem;
	}
</style>
