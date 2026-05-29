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
	import { t } from "svelte-i18n";

	let { event }: { event: EmoteEvent } = $props();
</script>

{#snippet userLink(actor?: User | null, by: boolean = true)}
	{#if actor && actor.mainConnection}
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
		<span class="text">
			{$t("dialogs.emote-events.upload.action")}
			{$t("words.by")}
			{@render userLink(event.actor)}
		</span>

	{:else if event.data.__typename === "EventEmoteDataProcess"}
		<Cpu />
		{#if event.data.event === "START"}
			<span class="text">{$t("dialogs.emote-events.process.start")}</span>
		{:else if event.data.event === "SUCCESS"}
			<span class="text">{$t("dialogs.emote-events.process.success")}</span>
		{:else if event.data.event === "FAIL"}
			<span class="text">{$t("dialogs.emote-events.process.fail")}</span>
		{:else if event.data.event === "CANCEL"}
			<span class="text">{$t("dialogs.emote-events.process.cancel")}</span>
		{/if}

	{:else if event.data.__typename === "EventEmoteDataChangeName"}
		<NotePencil />
		<span class="text">
			{$t("dialogs.emote-events.change_name.action")}
			{$t("words.from")} {event.data.oldName}
			{$t("words.to")} {event.data.newName}
			{$t("words.by")} {@render userLink(event.actor)}
		</span>

	{:else if event.data.__typename === "EventEmoteDataMerge"}
		<ArrowsMerge />
		<span class="text">
			{$t("dialogs.emote-events.merge.action")}
			{$t("words.with")} <a href="/emotes/{event.data.newEmote.id}">{event.data.newEmote.defaultName}</a>
			{$t("words.by")} {@render userLink(event.actor)}
		</span>

	{:else if event.data.__typename === "EventEmoteDataChangeOwner"}
		<Users />
		<span class="text">
			{$t("dialogs.emote-events.change_owner.action")}
			{$t("words.from")} {@render userLink(event.data.oldOwner, false)}
			{$t("words.to")} {@render userLink(event.data.newOwner, false)}
			{$t("words.by")} {@render userLink(event.actor)}
		</span>

	{:else if event.data.__typename === "EventEmoteDataChangeTags"}
		<NotePencil />
		<span class="text">
			{$t("dialogs.emote-events.change_tags.action")} {event.data.newTags}
			{$t("words.by")} {@render userLink(event.actor)}
		</span>

	{:else if event.data.__typename === "EventEmoteDataChangeFlags"}
		{#if !event.data.oldFlags.publicListed && event.data.newFlags.publicListed}
			<Check />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.approved_public.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if event.data.oldFlags.publicListed && !event.data.newFlags.publicListed}
			<X />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.removed_public.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if !event.data.oldFlags.approvedPersonal && event.data.newFlags.approvedPersonal}
			<Check />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.approved_personal.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if !event.data.oldFlags.deniedPersonal && event.data.newFlags.deniedPersonal}
			<X />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.rejected_personal.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if !event.data.oldFlags.defaultZeroWidth && event.data.newFlags.defaultZeroWidth}
			<StackSimple />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.added_overlaying.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if event.data.oldFlags.defaultZeroWidth && !event.data.newFlags.defaultZeroWidth}
			<StackSimple />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.removed_overlaying.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if !event.data.oldFlags.private && event.data.newFlags.private}
			<LockSimple />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.added_private.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>

		{:else if event.data.oldFlags.private && !event.data.newFlags.private}
			<LockSimpleOpen />
			<span class="text">
				{$t("dialogs.emote-events.change_flags.removed_private.action")}
				{$t("words.by")} {@render userLink(event.actor)}
			</span>
		{/if}

	{:else if event.data.__typename === "EventEmoteDataDelete"}
		<Trash />
		<span class="text">
			{$t("dialogs.emote-events.delete.action")}
			{$t("words.by")}
			{@render userLink(event.actor)}
		</span>
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
