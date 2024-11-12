<script lang="ts">
	import type { Snippet } from "svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";

	let {
		mode = $bindable("hidden"),
		title,
		width = 45,
		children,
		preview,
		buttons,
	}: {
		mode: DialogMode;
		title?: string;
		width?: number;
		children?: Snippet;
		preview?: Snippet;
		buttons: Snippet;
	} = $props();
</script>

<Dialog {width} bind:mode>
	<form class="layout">
		<div class="preview">
			<!-- <EmotePreview /> -->
			{@render preview?.()}
		</div>
		<div class="content">
			<h1>{title}</h1>
			{@render children?.()}
			<div class="buttons">
				{@render buttons()}
			</div>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1.5rem 1rem;

		display: flex;
		gap: 2rem;
	}

	.preview {
		align-self: center;
		min-width: 10rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
		align-items: center;
	}

	.content {
		flex-grow: 1;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
