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
			<div class="children">
				{@render children?.()}
			</div>
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

		max-width: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.children {
		flex-grow: 1;

		margin-right: -1rem;
		padding-right: 1rem;

		overflow-x: hidden;
		overflow-y: auto;
		scrollbar-gutter: stable;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;

		max-width: 100%;
		min-height: 1.2rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}

	@media (min-width: 961px) {
		.content {
			max-height: 80vh;
		}
	}
</style>
