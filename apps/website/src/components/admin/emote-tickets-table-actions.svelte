<script lang="ts">
	import { ArrowsMerge, Check, EyeSlash, Trash } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import type { ButtonOptions } from "./emote-ticket.svelte";

	let {
		buttonOptions = $bindable(),
		onaction,
	}: {
		buttonOptions: ButtonOptions;
		onaction: (action: string) => void;
	} = $props();
	function onClickStop(event: Event, action: string) {
		event.stopPropagation();
		onaction(action);
	}
</script>

<td class="shrink">
	<div class="buttons">
		{#if buttonOptions.merge}
			<Button onclick={(event) => onClickStop(event, "merge")}>
				{#snippet icon()}
					<ArrowsMerge style="transform: rotate(-90deg)" color="var(--admin-merge)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.delete}
			<Button onclick={(event) => onClickStop(event, "delete")}>
				{#snippet icon()}
					<Trash color="var(--danger)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.unlist}
			<Button onclick={(event) => onClickStop(event, "unlist")}>
				{#snippet icon()}
					<EyeSlash color="var(--admin-unlist)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.approve}
			<Button onclick={(event) => onClickStop(event, "approve")}>
				{#snippet icon()}
					<Check color="var(--approve)" />
				{/snippet}
			</Button>
		{/if}
	</div>
</td>

<style lang="scss">
	.buttons {
		display: flex;
		align-items: center;
	}
</style>
