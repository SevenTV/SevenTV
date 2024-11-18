<script lang="ts">
	import { FolderSimple } from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import { defaultEmoteSetDialogMode } from "$/lib/layout";
	import { defaultEmoteSet } from "$/lib/defaultEmoteSet";
	import { user } from "$/lib/auth";

	let emoteSetName = $derived(
		$defaultEmoteSet
			? $user?.editableEmoteSets.find((e) => e.id === $defaultEmoteSet)?.name
			: undefined,
	);
</script>

<Button secondary hideOnMobile onclick={() => ($defaultEmoteSetDialogMode = "shown")}>
	{#snippet icon()}
		<FolderSimple />
	{/snippet}
	{emoteSetName ?? "Default Set"}
</Button>
