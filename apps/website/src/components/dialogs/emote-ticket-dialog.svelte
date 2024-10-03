<script lang="ts">
	import {
		ArrowsMerge,
		Check,
		EyeSlash,
		Gavel,
		NotePencil,
		PaperPlaneRight,
		Trash,
	} from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Dialog from "./dialog.svelte";
	import type { DialogMode } from "./dialog.svelte";
	import TabLink from "../tab-link.svelte";
	import EmoteInfo from "../emotes/emote-info.svelte";

	export let mode: DialogMode;

	let tab: "activity" | "comments" = "activity";
</script>

<Dialog bind:mode width={42}>
	<form class="layout">
		<h1>Emote queue</h1>
		<hr />
		<div class="emote">
			<!-- <EmoteInfo>
				<Button secondary>
					<NotePencil slot="icon" />
					Edit
				</Button>
				<Button secondary>
					<ArrowsMerge style="transform: rotate(-90deg)" slot="icon" />
					Merge
				</Button>
				<Button secondary>
					<PaperPlaneRight slot="icon" />
					Transfer
				</Button>
				<Button secondary style="color: var(--danger)">
					<Gavel slot="icon" />
					Ban Owner
				</Button>
			</EmoteInfo> -->
		</div>
		<hr />
		<div class="tabs">
			<TabLink
				title="Activity Log (17)"
				matcher={() => tab === "activity"}
				on:click={() => (tab = "activity")}
			/>
			<TabLink
				title="Mod Comments (12)"
				matcher={() => tab === "comments"}
				on:click={() => (tab = "comments")}
			/>
		</div>
		{#if tab === "activity"}
			Activity
		{:else}
			Comments
		{/if}
		<hr />
		<div class="buttons">
			<Button>
				<Trash slot="icon" color="var(--danger)" />
			</Button>
			<Button>
				<EyeSlash slot="icon" color="var(--admin-unlist)" />
			</Button>
			<Button>
				<Check slot="icon" color="var(--admin-approve)" />
			</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 0.5rem;
	}

	hr {
		margin-inline: -1rem;
	}

	.emote {
		padding: 0.75rem 0;
	}

	.tabs {
		align-self: flex-start;

		display: flex;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}

	.buttons {
		display: flex;
		justify-content: flex-end;
		align-items: center;
	}
</style>
