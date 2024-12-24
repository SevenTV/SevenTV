<script lang="ts">
	import type { UserEvent, Paint, EmoteSet, Badge, User } from "$/gql/graphql";
	import {
		FolderSimple,
		IconContext,
		PaintBrush,
		Plugs,
		PlugsConnected,
		Plus,
		Trash,
	} from "phosphor-svelte";
	import moment from "moment/min/moment-with-locales";
	import FromNow from "$/components/from-now.svelte";
	import PaintComponent from "../paint.svelte";
	import BadgeComponent from "../badge.svelte";

	let { event }: { event: UserEvent } = $props();
</script>

{#snippet badge(badge?: Badge | null)}
	{#if badge}
		<BadgeComponent inline {badge} enableDialog />
		{badge.name}
	{:else}
		No Badge
	{/if}
{/snippet}

{#snippet paint(paint?: Paint | null)}
	{#if paint}
		<PaintComponent {paint} style="font-weight: 700; display: inline;" enableDialog>
			{paint.name.length > 0 ? paint.name : paint.id}
		</PaintComponent>
	{:else}
		No Paint
	{/if}
{/snippet}

{#snippet emoteSetLink(emoteSet?: EmoteSet | null)}
	{#if emoteSet}
		<a href="/emote-sets/{emoteSet.id}">{emoteSet.name}</a>
	{:else}
		<s>Deleted Set</s>
	{/if}
{/snippet}

{#snippet userLink(actor?: User | null, by: boolean = true)}
	{#if actor && actor.mainConnection}
		{#if by}
			by
		{/if}
		<a href="/users/{actor.id}" class="user-link" style:color={actor.highestRoleColor?.hex}>
			{actor.mainConnection.platformDisplayName}
		</a>
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
		{#if event.data.__typename === "EventUserDataCreate"}
			<Plus />
			<span class="text">User created {@render userLink(event.actor)}</span>
		{:else if event.data.__typename === "EventUserDataChangeActivePaint"}
			<PaintBrush />
			<span class="text">
				Changed active paint from
				{@render paint(event.data.oldPaint)}
				to
				{@render paint(event.data.newPaint)}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventUserDataChangeActiveBadge"}
			<PaintBrush />
			<span class="text">
				Changed active badge from
				{@render badge(event.data.oldBadge)}
				to
				{@render badge(event.data.newBadge)}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventUserDataChangeActiveEmoteSet"}
			<FolderSimple />
			<span class="text">
				Changed active emote set from
				{@render emoteSetLink(event.data.oldEmoteSet)}
				to
				{@render emoteSetLink(event.data.newEmoteSet)}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventUserDataAddConnection"}
			<PlugsConnected />
			<span class="text"
				>Added {event.data.addedPlatform} connection {@render userLink(event.actor)}</span
			>
		{:else if event.data.__typename === "EventUserDataRemoveConnection"}
			<Plugs />
			<span class="text"
				>Removed {event.data.removedPlatform} connection {@render userLink(event.actor)}</span
			>
		{:else if event.data.__typename === "EventUserDataDelete"}
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

		a {
			color: var(--text);
		}

		.time {
			grid-area: time;
			color: var(--text-light);
		}
	}
</style>
