<script module lang="ts">
	export interface ButtonOptions {
		merge: boolean;
		delete: boolean;
		unlist: boolean;
		approve: boolean;
	}
</script>

<script lang="ts">
	import {
		ArrowCounterClockwise,
		ArrowsMerge,
		Check,
		Clock,
		EyeSlash,
		Smiley,
		Trash,
		User,
	} from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import moment from "moment/min/moment-with-locales";
	import Flags from "../flags.svelte";
	import FromNow from "../from-now.svelte";
	import type { ModRequestsTicket } from "$/components/admin/report-tickets.svelte";
	import ResponsiveImage from "../responsive-image.svelte";

	let {
		buttonOptions = $bindable(),
		onclick,
		ticket = $bindable(),
		onaction,
		onrevertticketaction,
	}: {
		buttonOptions: ButtonOptions;
		onclick: () => void;
		ticket: ModRequestsTicket;
		onaction: (action: string) => void;
		onrevertticketaction: (ticket: ModRequestsTicket) => void;
	} = $props();
	let { emote } = ticket;
	function onClickStop(event: Event, action: string) {
		event.stopPropagation();
		onaction(action);
	}
</script>

<button class="emote-ticket" {onclick}>
	<div class="buttons">
		{#if ticket.isActioned}
			<Button
				big
				title="Revert action"
				onclick={async (event) => {
					event.stopPropagation();
					await onrevertticketaction(ticket);
				}}
			>
				{#snippet icon()}
					<ArrowCounterClockwise color="lightblue" />
				{/snippet}
			</Button>
		{/if}
		{#if !ticket.isActioned}
			{#if buttonOptions.approve}
				<Button onclick={(event) => onClickStop(event, "approve")}>
					{#snippet icon()}
						<Check color="var(--approve)" />
					{/snippet}
				</Button>
			{/if}
			{#if buttonOptions.unlist}
				<Button onclick={(event) => onClickStop(event, "unlist")}>
					{#snippet icon()}
						<EyeSlash color="var(--admin-unlist)" />
					{/snippet}
				</Button>
			{/if}
			{#if buttonOptions.delete}
				<Button onclick={(event) => onClickStop(event, "delete")}>
					{#snippet icon()}
						<Trash color="var(--danger)" />
					{/snippet}
				</Button>
			{/if}
			{#if buttonOptions.merge}
				<Button onclick={(event) => onClickStop(event, "merge")}>
					{#snippet icon()}
						<ArrowsMerge style="transform: rotate(-90deg)" color="var(--admin-merge)" />
					{/snippet}
				</Button>
			{/if}
		{/if}
	</div>
	<div class="emote-preivew">
		<ResponsiveImage images={ticket.emote.images} />
	</div>
	<div class="info">
		<a class="emote-name field" href="/emotes/{emote.id}" title={emote.defaultName}>
			<Smiley />
			{emote.defaultName}
		</a>
		<a
			class="username field owner"
			href="/users/{emote.owner?.id}"
			title={emote.owner?.mainConnection?.platformUsername}
		>
			<User />
			{emote.owner?.mainConnection?.platformUsername}
		</a>
		<div class="field from-now">
			<Clock />
			<FromNow date={moment(ticket.message.created_at)} />
		</div>
		{#if emote.tags && emote.tags.length > 0}
			{#if emote.tags.length > 3}
				<Flags flags={emote.tags.slice(0, 3).concat(["..."])} />
			{:else}
				<Flags flags={emote.tags} />
			{/if}
		{/if}
	</div>
</button>

<style lang="scss">
	.emote-ticket {
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
		padding: 0.75rem;
		display: flex;
		gap: 0.5rem;
	}

	.username {
		white-space: nowrap !important;
		overflow: hidden !important;
		text-overflow: ellipsis !important;
		display: block !important;
		max-width: 12ch !important;
		text-align: left;
	}

	.emote-name {
		white-space: nowrap !important;
		overflow: hidden !important;
		text-overflow: ellipsis !important;
		display: block !important;
		max-width: 12ch !important;
		text-align: left;
	}

	.emote-preivew {
		display: flex;
		align-items: center;
		position: relative;
		max-width: 200px;
		justify-content: center;
		max-height: 9.5rem;
		height: 100%;
		padding: 0.75rem;
		width: 100%;

		align-items: center;

		> :global(picture > img) {
			object-fit: contain;
			height: 100%;
			width: 100%;
		}

		> :global(picture) {
			flex-grow: 1;
			width: 100%;
			height: 100%;
		}
	}

	.buttons {
		display: flex;
		flex-direction: column;
	}

	.info {
		flex-grow: 1;
		margin-block: auto;
		margin-left: 0.5rem;

		display: grid;
		grid-template-columns: auto 1fr;
		align-items: center;
		column-gap: 0.5rem;
		row-gap: 0.75rem;
	}

	.field {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		text-overflow: ellipsis;
		color: var(--text);
		width: 100px;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.from-now {
		justify-self: end;
		gap: 0.3rem;

		color: var(--text-light);
		text-align: right;
	}
</style>
