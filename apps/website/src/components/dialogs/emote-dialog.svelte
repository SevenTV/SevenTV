<script lang="ts">
	import type { Snippet } from "svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import type { Emote } from "$/gql/graphql";

	interface Props {
		mode: DialogMode;
		data: Emote;
		title: string;
		width?: number;
		children?: Snippet;
		preview?: Snippet;
		buttons: Snippet;
	}

	let {
		mode = $bindable("hidden"),
		data,
		title,
		width = 45,
		children,
		preview,
		buttons,
	}: Props = $props();
</script>

<Dialog {width} bind:mode>
	<form class="layout">
		<div class="preview">
			<EmotePreview {data} emoteOnly />
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
		flex-wrap: wrap;
		gap: 2rem;

		height: 100%;
	}

	.preview {
		flex-grow: 1;
		min-width: 8rem;
		align-self: center;

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
