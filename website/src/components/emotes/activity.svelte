<script lang="ts">
	import { IconContext, Plus, NotePencil, Check, X } from "phosphor-svelte";

	export let activities = [
		{
			kind: "reject",
			time: "1 hour ago",
			message: [
				{ text: "forsen", href: "/user/forsen", bold: true },
				{ text: "rejected" },
				{ text: "AlienDance", bold: true },
				{ text: "for personal use" },
			],
		},
		{
			kind: "modify",
			time: "1 hour ago",
			message: [
				{ text: "ayyybubu", href: "/user/ayyybubu", bold: true },
				{ text: "renamed" },
				{ text: "AlienPls", bold: true },
				{ text: "to" },
				{ text: "AlienDance", bold: true },
			],
		},
		{
			kind: "approve",
			time: "1 hour ago",
			message: [
				{ text: "forsen", href: "/user/forsen", bold: true },
				{ text: "approved" },
				{ text: "AlienPls", bold: true },
				{ text: "for public listing" },
			],
		},
		{
			kind: "create",
			time: "1 hour ago",
			message: [
				{ text: "ayyybubu", href: "/user/ayyybubu", bold: true },
				{ text: "created" },
				{ text: "AlienPls", bold: true },
			],
		},
	];
</script>

{#each activities as activity, index}
	<div class="event">
		<IconContext values={{ style: "margin: 0 0.5rem", size: "1rem" }}>
			{#if activity.kind === "reject"}
				<X color="var(--error)" />
			{:else if activity.kind === "modify"}
				<NotePencil />
			{:else if activity.kind === "approve"}
				<Check color="var(--secondary)" />
			{:else}
				<Plus color="var(--secondary)" />
			{/if}
		</IconContext>

		<div class="event-message">
			<div class="event-text">
				{#each activity.message as item, i}
					{#if item.href}
						<a href={item.href} class={item.bold ? "bold-text" : ""}>{item.text}</a>
					{:else}
						<span class={item.bold ? "bold-text" : ""}>{item.text}</span>
					{/if}
					{#if i !== activity.message.length - 1}
						<span> </span>
					{/if}
				{/each}
			</div>
			<span class="time">{activity.time}</span>
		</div>
	</div>
	{#if index !== activities.length - 1}
		<hr />
	{/if}
{/each}

<style lang="scss">
	.bold-text {
		font-weight: 700;
		color: var(--text);
	}

	a:hover {
		text-decoration: none;
	}

	.event {
		display: flex;
		margin: 1rem 0;

		.event-message {
			font-size: 0.813rem;

			.time {
				font-size: 0.75rem;
				font-weight: 500;
				color: var(--text-lighter);
			}
		}
	}
</style>
