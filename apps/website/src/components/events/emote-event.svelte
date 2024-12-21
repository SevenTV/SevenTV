<script lang="ts">
	import type { EmoteEvent, User } from "$/gql/graphql";
	import {
		ArrowsMerge,
		Check,
		Cpu,
		IconContext,
		LockSimple,
		LockSimpleOpen,
		NotePencil,
		Plus,
		StackSimple,
		Trash,
		Users,
		X,
	} from "phosphor-svelte";
	import moment from "moment/min/moment-with-locales";
	import FromNow from "$/components/from-now.svelte";

	let { event }: { event: EmoteEvent } = $props();
</script>

{#snippet userLink(actor?: User | null, by: boolean = true)}
	{#if actor && actor.mainConnection}
		{#if by}
			by
		{/if}
		<a href="/users/{actor.id}" class="user-link" style:color={actor.highestRoleColor?.hex}
			>{actor.mainConnection.platformDisplayName}</a
		>
	{/if}
{/snippet}

<IconContext
	values={{
		style: "grid-area: icon; margin: 0 0.5rem;",
		size: 1.2 * 16,
		color: "var(--primary)",
	}}
>
	<div class="event">
		{#if event.data.__typename === "EventEmoteDataUpload"}
			<Plus />
			<span class="text">Uploaded {@render userLink(event.actor)}</span>
		{:else if event.data.__typename === "EventEmoteDataProcess"}
			<Cpu />
			{#if event.data.event === "START"}
				<span class="text">Started a new processing job</span>
			{:else if event.data.event === "SUCCESS"}
				<span class="text">Successfully finished processing</span>
			{:else if event.data.event === "FAIL"}
				<span class="text">Failed processing</span>
			{:else if event.data.event === "CANCEL"}
				<span class="text">Cancelled processing job</span>
			{/if}
		{:else if event.data.__typename === "EventEmoteDataChangeName"}
			<NotePencil />
			<span class="text">
				Renamed from {event.data.oldName} to {event.data.newName}
				{@render userLink(event.actor)}</span
			>
		{:else if event.data.__typename === "EventEmoteDataMerge"}
			<ArrowsMerge />
			<span class="text"
				>Merged with {event.data.newEmote.defaultName} {@render userLink(event.actor)}</span
			>
		{:else if event.data.__typename === "EventEmoteDataChangeOwner"}
			<Users />
			<span class="text"
				>Transferred ownership from {@render userLink(event.data.oldOwner, false)} to {@render userLink(
					event.data.newOwner,
					false,
				)}
				{@render userLink(event.actor)}</span
			>
		{:else if event.data.__typename === "EventEmoteDataChangeTags"}
			<NotePencil />
			<span class="text">Set tags to {event.data.newTags} {@render userLink(event.actor)}</span>
		{:else if event.data.__typename === "EventEmoteDataChangeFlags"}
			{#if !event.data.oldFlags.publicListed && event.data.newFlags.publicListed}
				<Check />
				<span class="text">Approved for public listing {@render userLink(event.actor)}</span>
			{:else if event.data.oldFlags.publicListed && !event.data.newFlags.publicListed}
				<X />
				<span class="text">Removed from public listing {@render userLink(event.actor)}</span>
			{:else if !event.data.oldFlags.approvedPersonal && event.data.newFlags.approvedPersonal}
				<Check />
				<span class="text">Approved for personal use {@render userLink(event.actor)}</span>
			{:else if !event.data.oldFlags.deniedPersonal && event.data.newFlags.deniedPersonal}
				<X />
				<span class="text">Rejected for personal use {@render userLink(event.actor)}</span>
			{:else if !event.data.oldFlags.defaultZeroWidth && event.data.newFlags.defaultZeroWidth}
				<StackSimple />
				<span class="text">Added flag overlaying {@render userLink(event.actor)}</span>
			{:else if event.data.oldFlags.defaultZeroWidth && !event.data.newFlags.defaultZeroWidth}
				<StackSimple />
				<span class="text">Removed flag overlaying {@render userLink(event.actor)}</span>
			{:else if !event.data.oldFlags.private && event.data.newFlags.private}
				<LockSimple />
				<span class="text">Added flag private {@render userLink(event.actor)}</span>
			{:else if event.data.oldFlags.private && !event.data.newFlags.private}
				<LockSimpleOpen />
				<span class="text">Removed flag private {@render userLink(event.actor)}</span>
			{/if}
		{:else if event.data.__typename === "EventEmoteDataDelete"}
			<Trash />
			<span class="text">Deleted {@render userLink(event.actor)}</span>
		{/if}
		<span class="time">
			<FromNow date={moment(event.createdAt)} />
		</span>
	</div>
</IconContext>

<style lang="scss">
	.event {
		display: grid;
		grid-template-areas: "icon text" ". time";
		justify-content: start;
		align-items: center;
		row-gap: 0.5rem;
		margin: 1rem 0;

		font-size: 0.75rem;
		font-weight: 500;

		.text {
			grid-area: text;
		}

		.user-link {
			color: var(--text);
		}

		.time {
			grid-area: time;
			color: var(--text-light);
		}
	}
</style>
