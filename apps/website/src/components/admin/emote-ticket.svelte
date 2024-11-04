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
		ArrowsMerge,
		Check,
		Clock,
		EyeSlash,
		SealCheck,
		Smiley,
		Trash,
		User,
	} from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import CountryFlag from "../country-flag.svelte";
	import moment from "moment/min/moment-with-locales";
	import Flags from "../flags.svelte";
	import FromNow from "../from-now.svelte";

	let {
		buttonOptions = $bindable(),
		onclick,
	}: { buttonOptions: ButtonOptions; onclick: () => void } = $props();
</script>

<button class="emote-ticket" {onclick}>
	<div class="buttons">
		{#if buttonOptions.approve}
			<Button>
				{#snippet icon()}
					<Check color="var(--admin-approve)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.unlist}
			<Button>
				{#snippet icon()}
					<EyeSlash color="var(--admin-unlist)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.delete}
			<Button>
				{#snippet icon()}
					<Trash color="var(--danger)" />
				{/snippet}
			</Button>
		{/if}
		{#if buttonOptions.merge}
			<Button>
				{#snippet icon()}
					<ArrowsMerge style="transform: rotate(-90deg)" color="var(--admin-merge)" />
				{/snippet}
			</Button>
		{/if}
	</div>
	<!-- <EmotePreview
		emoteOnly
		style="width: auto; align-self: center; flex-shrink: 0; pointer-events: none"
	/> -->
	<div class="info">
		<div class="field">
			<Smiley />
			EmoteName
		</div>
		<CountryFlag code="fr" name="France" height={1.2 * 16} style="justify-self: end" />
		<a class="field" href="/users/username">
			<User />
			Username
			<SealCheck weight="fill" color="var(--store)" />
		</a>
		<div class="field from-now">
			<Clock />
			<FromNow date={moment("2024-03-18T19:30:00")} />
		</div>
		<Flags
			flags={["overlaying", "lorem", "ipsum", "dolor", "sit", "amet", "consectetur"]}
			style="grid-column: span 2"
		/>
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

		color: var(--text);
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
