<script lang="ts">
	import EmoteTabs from "$/components/layout/emote-tabs.svelte";
	import { Check, IconContext, NotePencil, Plus, X } from "phosphor-svelte";
	import type { LayoutData } from "../$types";
	import { t } from "svelte-i18n";
	import moment from "moment/min/moment-with-locales";
	import FromNow from "$/components/from-now.svelte";

	export let data: LayoutData;

	const activities = [
		{
			kind: "reject",
			user: "ayyybubu",
			emote: "AlienPls",
		},
		{
			kind: "modify",
			user: "ayyybubu",
			emote: "AlienPls",
		},
		{
			kind: "approve",
			user: "ayyybubu",
			emote: "AlienPls",
		},
		{
			kind: "create",
			user: "ayyybubu",
			emote: "AlienPls",
		},
	];
</script>

<div class="navigation">
	<EmoteTabs id={data.id} />
</div>
<div class="activities">
	{#each activities as activity, index}
		<div class="event">
			<IconContext
				values={{
					style: "grid-area: icon; margin: 0 0.5rem;",
					size: "1.2rem",
					color: "var(--primary)",
				}}
			>
				{#if activity.kind === "reject"}
					<X />
				{:else if activity.kind === "modify"}
					<NotePencil />
				{:else if activity.kind === "approve"}
					<Check />
				{:else}
					<Plus />
				{/if}
			</IconContext>

			<span class="text">
				{$t(`activities.${activity.kind}`, {
					values: { user: activity.user, emote: activity.emote },
				})}
			</span>
			<span class="time"><FromNow date={moment()} /></span>
		</div>
		{#if index !== activities.length - 1}
			<hr />
		{/if}
	{/each}
</div>

<style lang="scss">
	.navigation {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.activities {
		margin-top: 1.5rem;
	}

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

		.time {
			grid-area: time;
			color: var(--text-light);
		}
	}

	hr {
		color: var(--border-active);
	}
</style>
