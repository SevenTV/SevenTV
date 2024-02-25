<script lang="ts">
	import { Plus, NotePencil, Check, X } from "phosphor-svelte";

	export let event = {
		type: "created",
		author: "eventAuthor",
		oldName: "oldName",
		emoteName: "emoteName",
		time: "1 hour ago",
	};
</script>

<div class="event">
	{#if event.type === "renamed"}
		<NotePencil style="margin: 0 0.5rem" weight="bold" size="1rem" />
		<div class="event-message">
			<div class="event-text">
				<a href="/user/ayyybubu" class="user-name">
					{event.author}
				</a>
				<span> renamed </span>
				<span class="user-name">
					{event.oldName}
				</span>
				<span> to </span>
				<span class="emote-name">
					{event.emoteName}
				</span>
			</div>
			<span class="time">
				{event.time}
			</span>
		</div>
	{:else if event.type === "approved"}
		<Check style="margin: 0 0.5rem" weight="bold" color="var(--primary)" size="1rem" />
		<div class="event-message">
			<div class="event-text">
				<a href="/user/forsen" class="user-name">
					{event.author}
				</a>
				<span> approved </span>
				<span class="emote-name">
					{event.emoteName}
				</span>
				<span> for public listing </span>
			</div>
			<span class="time">
				{event.time}
			</span>
		</div>
	{:else if event.type === "rejected-personal"}
		<X style="margin: 0 0.5rem" weight="bold" color="var(--error)" size="1rem" />
		<div class="event-message">
			<div class="event-text">
				<a href="/user/forsen" class="user-name">
					{event.author}
				</a>
				<span> rejected personal use for </span>
				<span class="emote-name">
					{event.emoteName}
				</span>
			</div>
			<span class="time">
				{event.time}
			</span>
		</div>
	{:else}
		<Plus style="margin: 0 0.5rem" weight="bold" color="var(--primary)" size="1rem" />
		<div class="event-message">
			<div class="event-text">
				<a href="/user/ayyybubu" class="user-name">
					{event.author}
				</a>
				<span> created </span>
				<span class="emote-name">
					{event.emoteName}
				</span>
			</div>
			<span class="time">
				{event.time}
			</span>
		</div>
	{/if}
</div>

<style lang="scss">
	.user-name,
	.emote-name {
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
