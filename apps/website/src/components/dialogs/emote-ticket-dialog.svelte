<script lang="ts">
	import { Check, EyeSlash, Trash } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import TabLink from "../tab-link.svelte";

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();

	let tab: "activity" | "comments" = $state("activity");
</script>

<Dialog bind:mode width={42}>
	<form class="layout">
		<h1>Emote queue</h1>
		<hr />
		<div class="emote">
			<!-- <EmoteInfo>
				<Button secondary>
					<NotePencil />
					Edit
				</Button>
				<Button secondary>
					<ArrowsMerge style="transform: rotate(-90deg)" />
					Merge
				</Button>
				<Button secondary>
					<PaperPlaneRight />
					Transfer
				</Button>
				<Button secondary style="color: var(--danger)">
					<Gavel />
					Ban Owner
				</Button>
			</EmoteInfo> -->
		</div>
		<hr />
		<div class="tabs">
			<TabLink
				title="Activity Log (17)"
				matcher={() => tab === "activity"}
				onclick={() => (tab = "activity")}
			/>
			<TabLink
				title="Mod Comments (12)"
				matcher={() => tab === "comments"}
				onclick={() => (tab = "comments")}
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
				{#snippet icon()}
					<Trash color="var(--danger)" />
				{/snippet}
			</Button>
			<Button>
				{#snippet icon()}
					<EyeSlash color="var(--admin-unlist)" />
				{/snippet}
			</Button>
			<Button>
				{#snippet icon()}
					<Check color="var(--approve)" />
				{/snippet}
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
