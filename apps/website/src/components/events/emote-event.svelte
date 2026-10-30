<script lang="ts">
	import type { EmoteEvent, EmoteFlags, User } from "$/gql/graphql";
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
	import UserProfilePicture from "$/components/user-profile-picture.svelte";
	import UserName from "$/components/user-name.svelte";

	let { event }: { event: EmoteEvent } = $props();

	// One semantic color per event, reused for both the icon and the left stripe,
	// using the same theme variables already used for this purpose in the admin UI
	// (--approve, --danger, --rename, --admin-merge, --admin-unlist).
	function flagsColor(oldFlags: EmoteFlags, newFlags: EmoteFlags): string {
		if (!oldFlags.publicListed && newFlags.publicListed) return "var(--approve)";
		if (oldFlags.publicListed && !newFlags.publicListed) return "var(--danger)";
		if (!oldFlags.approvedPersonal && newFlags.approvedPersonal) return "var(--approve)";
		if (!oldFlags.deniedPersonal && newFlags.deniedPersonal) return "var(--danger)";
		if (!oldFlags.defaultZeroWidth && newFlags.defaultZeroWidth) return "var(--rename)";
		if (oldFlags.defaultZeroWidth && !newFlags.defaultZeroWidth) return "var(--rename)";
		if (!oldFlags.private && newFlags.private) return "var(--admin-unlist)";
		if (oldFlags.private && !newFlags.private) return "var(--approve)";
		return "var(--primary)";
	}

	let color = $derived.by(() => {
		switch (event.data.__typename) {
			case "EventEmoteDataProcess":
				switch (event.data.event) {
					case "SUCCESS":
						return "var(--approve)";
					case "FAIL":
						return "var(--danger)";
					case "CANCEL":
						return "var(--admin-unlist)";
					case "START":
						return "var(--admin-unlist)";
					default:
						return "var(--text-light)";
				}
			case "EventEmoteDataUpload":
			case "EventEmoteDataChangeName":
			case "EventEmoteDataChangeTags":
				return "var(--rename)";
			case "EventEmoteDataMerge":
				return "var(--admin-merge)";
			case "EventEmoteDataChangeFlags":
				return flagsColor(event.data.oldFlags, event.data.newFlags);
			case "EventEmoteDataDelete":
				return "var(--danger)";
			default:
				return "var(--primary)";
		}
	});
</script>

{#snippet userLink(actor?: User | null, by: boolean = true)}
	{#if actor && actor.mainConnection}
		<a href="/users/{actor.id}" class="user-link">
			<UserProfilePicture user={actor} size={24} />
			<span class="username" style:color={actor.highestRoleColor?.hex}>
				<UserName user={actor} />
			</span>
		</a>
	{/if}
{/snippet}

<IconContext values={{ style: "grid-area: icon; margin: 0 0.5rem;", size: 1.2 * 16, color }}>
	<div class="event" style:border-left-color={color}>
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
				{$t("words.from")}
				{event.data.oldName}
				{$t("words.to")}
				{event.data.newName}
				{$t("words.by")}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventEmoteDataMerge"}
			<ArrowsMerge />
			<span class="text">
				{$t("dialogs.emote-events.merge.action")}
				{$t("words.with")}
				<a href="/emotes/{event.data.newEmote.id}">{event.data.newEmote.defaultName}</a>
				{$t("words.by")}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventEmoteDataChangeOwner"}
			<Users />
			<span class="text">
				{$t("dialogs.emote-events.change_owner.action")}
				{$t("words.from")}
				{@render userLink(event.data.oldOwner, false)}
				{$t("words.to")}
				{@render userLink(event.data.newOwner, false)}
				{$t("words.by")}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventEmoteDataChangeTags"}
			<NotePencil />
			<span class="text">
				{$t("dialogs.emote-events.change_tags.action")}
				{event.data.newTags}
				{$t("words.by")}
				{@render userLink(event.actor)}
			</span>
		{:else if event.data.__typename === "EventEmoteDataChangeFlags"}
			{#if !event.data.oldFlags.publicListed && event.data.newFlags.publicListed}
				<Check />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.approved_public.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if event.data.oldFlags.publicListed && !event.data.newFlags.publicListed}
				<X />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.removed_public.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if !event.data.oldFlags.approvedPersonal && event.data.newFlags.approvedPersonal}
				<Check />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.approved_personal.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if !event.data.oldFlags.deniedPersonal && event.data.newFlags.deniedPersonal}
				<X />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.rejected_personal.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if !event.data.oldFlags.defaultZeroWidth && event.data.newFlags.defaultZeroWidth}
				<StackSimple />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.added_overlaying.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if event.data.oldFlags.defaultZeroWidth && !event.data.newFlags.defaultZeroWidth}
				<StackSimple />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.removed_overlaying.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if !event.data.oldFlags.private && event.data.newFlags.private}
				<LockSimple />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.added_private.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
				</span>
			{:else if event.data.oldFlags.private && !event.data.newFlags.private}
				<LockSimpleOpen />
				<span class="text">
					{$t("dialogs.emote-events.change_flags.removed_private.action")}
					{$t("words.by")}
					{@render userLink(event.actor)}
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
			<FromNow date={moment(event.createdAt)} showExact />
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
		margin: 0.75rem 0;
		padding-left: 0.75rem;

		border-left: 3px solid transparent;

		font-size: 0.75rem;
		font-weight: 500;

		.text {
			grid-area: text;
		}

		.user-link {
			display: inline-flex;
			align-items: center;
			gap: 0.35rem;
			vertical-align: middle;
			color: var(--text);
			text-decoration: none;

			&:hover .username,
			&:focus-visible .username {
				text-decoration: underline;
			}
		}

		.username {
			font-weight: 600;
		}

		.time {
			grid-area: time;
			color: var(--text-light);
		}
	}
</style>
