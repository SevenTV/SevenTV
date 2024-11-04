<script lang="ts">
	import { ArrowsMerge, Check, EyeSlash, Trash } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import type { ButtonOptions } from "./emote-ticket.svelte";

	let { buttonOptions = $bindable() }: { buttonOptions: ButtonOptions } = $props();
</script>

<td class="shrink">
	<div class="buttons">
		{#if buttonOptions.merge}
			<Button>
				{#snippet icon()}
					<ArrowsMerge style="transform: rotate(-90deg)" color="var(--admin-merge)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.delete}
			<Button>
				{#snippet icon()}
					<Trash color="var(--danger)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.unlist}
			<Button>
				{#snippet icon()}
					<EyeSlash color="var(--admin-unlist)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.approve}
			<Button>
				{#snippet icon()}
					<Check color="var(--admin-approve)" />
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
