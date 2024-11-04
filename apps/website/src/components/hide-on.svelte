<script lang="ts">
	import type { Snippet } from "svelte";

	let {
		mobile = false,
		desktop = false,
		children,
	}: { mobile?: boolean; desktop?: boolean; children: Snippet } = $props();
</script>

<div class="hide-on" class:hide-on-mobile={mobile} class:hide-on-desktop={desktop}>
	{@render children()}
</div>

<style lang="scss">
	.hide-on {
		display: contents;
	}
</style>
