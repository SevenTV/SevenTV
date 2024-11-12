<script lang="ts">
	import type { ButtonOptions } from "./emote-ticket.svelte";
	import EmoteTicketsButtonOptions from "./emote-tickets-button-options.svelte";

	let {
		actionsPosition = $bindable(),
		buttonOptions = $bindable(),
	}: { actionsPosition?: "left" | "right"; buttonOptions: ButtonOptions } = $props();
</script>

<th class="shrink">
	<div class="buttons">
		<EmoteTicketsButtonOptions bind:buttonOptions bind:actionsPosition />
	</div>
</th>

<style lang="scss">
	.buttons {
		display: flex;
		align-items: center;
	}
</style>
