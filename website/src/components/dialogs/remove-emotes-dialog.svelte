<script lang="ts">
	import { Layout } from "$/lib/stores";
	import EmoteContainer from "../emote-container.svelte";
	import EmotePreview from "../emote-preview.svelte";
	import Button from "../input/button.svelte";
	import Dialog, { DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	export let mode: DialogMode = DialogMode.Hidden;
</script>

<Dialog width={35} bind:mode>
	<form class="layout">
		<h1>{$t("dialogs.remove_emote.title", { values: { set: "Cat Emotes" } })}</h1>
		<hr />
		<EmoteContainer layout={Layout.SmallGrid} style="max-height: 11rem" scrollable>
			{#each Array(100) as _, i}
				<EmotePreview index={i} emoteOnly />
			{/each}
		</EmoteContainer>
		<div class="buttons">
			<Button style="color: var(--danger)" submit>{$t("labels.remove")}</Button>
			<Button secondary on:click={() => (mode = DialogMode.Hidden)}>{$t("labels.cancel")}</Button>
		</div>
	</form>
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

	.buttons {
		display: flex;
		justify-content: flex-end;
		align-items: center;
		gap: 0.5rem;
	}
</style>
