<script lang="ts">
	import { FolderSimple } from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import { defaultEmoteSetDialogMode } from "$/lib/layout";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { user } from "$/lib/auth";
	import { editableEmoteSets } from "$/lib/emoteSets";

	let emoteSetName = $derived(
		$defaultEmoteSet ? $editableEmoteSets.find((e) => e.id === $defaultEmoteSet)?.name : undefined,
	);
</script>

{#if $user}
	<Button secondary onclick={() => ($defaultEmoteSetDialogMode = "shown")}>
		{#snippet icon()}
			<FolderSimple />
		{/snippet}
		{emoteSetName ?? "Default Set"}
	</Button>
{/if}
