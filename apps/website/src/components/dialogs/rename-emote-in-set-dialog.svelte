<script lang="ts">
	import type { Emote, EmoteSetEmote } from "$/gql/graphql";
	import { renameEmoteInSet } from "$/lib/setMutations";
	import { PencilSimple } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import TextInput from "../input/text-input.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
		data: Emote | EmoteSetEmote;
		emoteInSet?: EmoteSetEmote;
		emoteSetName?: string;
		emoteSetId?: string;
		resetComponent?: () => void;
		hideFromRenameDialog?: () => void;
	}

	let {
		mode = $bindable("hidden"),
		data = $bindable(),
		emoteInSet = $bindable(),
		emoteSetId = $bindable(),
		emoteSetName = $bindable(),
		resetComponent,
		hideFromRenameDialog = $bindable(),
	}: Props = $props();

	function isEmoteSetEmote(d: Emote | EmoteSetEmote | undefined): d is EmoteSetEmote {
		return !!d && (d as EmoteSetEmote).emote !== undefined;
	}

	// hacky way for local checking cuz sometimes it doesn't return EmoteSetEmote fully because of query reactivity
	// TODO: fix this later ASAP!
	function isEmoteSetHalfEmote(d: Emote | EmoteSetEmote | undefined): d is EmoteSetEmote {
		return !!d && (d as EmoteSetEmote) !== undefined;
	}

	let targetEmoteName = $state("");

	const initialName = $derived.by(() => {
		if (!data) return "";
		if (emoteInSet?.alias) return emoteInSet.alias;
		if (isEmoteSetEmote(data)) return data.alias ?? data.emote?.defaultName ?? "";
		if (isEmoteSetHalfEmote(data)) return data.alias;
		return (data as Emote).defaultName ?? "";
	});

	const resolvedEmote = $derived(isEmoteSetEmote(data) ? data.emote : (data as Emote));
	$effect(() => {
		if (!data) return;
		if (emoteInSet?.alias) {
			targetEmoteName = emoteInSet.alias;
		} else if (isEmoteSetEmote(data)) {
			targetEmoteName = data.alias ?? data.emote?.defaultName ?? "";
		} else {
			targetEmoteName = data.defaultName ?? "";
		}
		targetEmoteName = initialName;
	});

	let loading = $state(false);

	// used try catch for future mutation/query functions remake throwing errors
	async function click() {
		if (!resolvedEmote) return;
		loading = true;
		try {
			await renameEmoteInSet(emoteSetId ?? "", resolvedEmote.id, targetEmoteName, initialName);
			resetComponent?.();
			mode = "hidden";
			hideFromRenameDialog?.();
		} catch (error) {
			console.error("Failed to rename emote:", error);
		} finally {
			loading = false;
		}
	}
</script>

{#snippet loadingSpinner()}
	<Spinner />
{/snippet}

<Dialog width={35} bind:mode>
	<div class="rename-emote-dialog">
		{#if emoteSetName}
			<h1 class="emote-set-name">
				{$t("dialogs.rename_emote.title", { values: { set: emoteSetName } })}
			</h1>
		{/if}
		<p>{$t("dialogs.rename_emote.confirmation_message", { values: { set: emoteSetName } })}</p>
		<TextInput
			placeholder={$t("dialogs.rename_emote.input_placeholder")}
			bind:value={targetEmoteName}
		>
			{#snippet icon()}
				<PencilSimple />
			{/snippet}
		</TextInput>
		<div class="button-row">
			<Button
				style="color: var(--danger)"
				disabled={loading}
				onclick={click}
				submit
				icon={loading ? loadingSpinner : undefined}
			>
				{$t("dialogs.rename_emote.confirm")}
			</Button>
			<Button
				secondary
				onclick={() => {
					mode = "hidden";
				}}>{$t("labels.cancel")}</Button
			>
		</div>
	</div>
</Dialog>

<style lang="scss">
	.rename-emote-dialog {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		padding: 2rem;

		.input-text {
			padding: 0.5rem 0.75rem;
			font-size: 1rem;
			border: 1px solid var(--border);
			border-radius: 6px;
			background: var(--background-secondary);
			color: var(--text);
			outline: none;
			transition: border-color 0.2s;
			width: 100%;
			box-sizing: border-box;

			&:focus {
				border-color: var(--primary);
			}
		}

		.button-row {
			display: flex;
			gap: 0.75rem;
			justify-content: flex-end;
		}
	}
</style>
