<script lang="ts">
	import { Warning } from "phosphor-svelte";
import Dialog, { type DialogMode } from "./dialog.svelte";

	interface Props {
		mode: DialogMode;
		error?: string;
	}

	let { mode = $bindable("hidden"), error = "An error occured" }: Props = $props();
</script>

<Dialog width={30} bind:mode>
	<div class="layout">
		<h1>Error</h1>
		<hr />
		<p>{error}</p>
	</div>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}
</style>
