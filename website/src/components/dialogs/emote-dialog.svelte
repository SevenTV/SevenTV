<script lang="ts">
	import EmotePreview from "../emote-preview.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
	export let title: string;
	export let width: number = 45;
</script>

<Dialog {width} bind:mode>
	<form class="layout">
		<div class="preview">
			<EmotePreview />
			<slot name="preview" />
		</div>
		<div class="content">
			<h1>{title}</h1>
			<slot />
			<div class="buttons">
				<slot name="buttons" />
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
