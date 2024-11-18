<script lang="ts">
	import EmoteSetPicker from "../emote-set-picker.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();
</script>

<Dialog width={35} bind:mode>
	<div class="layout">
		<h1>{$t("dialogs.copy_emotes.title")}</h1>
		<hr />
		<!-- <EmoteContainer layout={"small-grid"} style="max-height: 11rem" scrollable>
			{#each Array(100) as _, i}
				<EmotePreview index={i} emoteOnly />
			{/each}
		</EmoteContainer> -->
		<!-- <EmoteSetPicker emoteId={data.id} /> -->
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
