<script lang="ts">
	import { Layout } from "$/lib/stores";
	import EmoteContainer from "../emote-container.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog width={35} bind:mode>
	<div class="layout">
		<h1>Copy selected emotes to</h1>
		<hr />
		<EmoteContainer layout={Layout.SmallGrid} style="max-height: 11rem" scrollable>
			{#each Array(100) as _, i}
				<EmotePreview index={i} emoteOnly />
			{/each}
		</EmoteContainer>
		<EmoteSetPicker />
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
