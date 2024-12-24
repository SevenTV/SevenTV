<script lang="ts">
	import { graphql } from "$/gql";
	import { gqlClient } from "$/lib/gql";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";

	let { mode = $bindable(), userId }: { mode: DialogMode; userId: string } = $props();

	let loading = $state(false);

	let expiration = $state<string>();

	let token = $state<string>();

	async function submit(e: SubmitEvent) {
		e.preventDefault();

		if (!expiration) return;

		loading = true;

		const res = await gqlClient().mutation(
			graphql(`
				mutation AdminCreateSession($userId: Id!, $expiresAt: DateTime!) {
					userSessions {
						create(userId: $userId, expiresAt: $expiresAt)
					}
				}
			`),
			{ userId, expiresAt: new Date(expiration) },
		);

		loading = false;
		token = res.data?.userSessions.create;
	}

	function reset() {
		mode = "hidden";
		token = undefined;
	}
</script>

<Dialog bind:mode>
	<form onsubmit={submit} class="layout">
		<h1>Create Session</h1>
		<hr />
		<label>
			Expiration date in {Intl.DateTimeFormat().resolvedOptions().timeZone} time
			<input type="datetime-local" required bind:value={expiration} />
		</label>
		{#snippet loadingSpinner()}
			<Spinner />
		{/snippet}
		{#if token}
			<label>
				Token
				<textarea readonly class="token">{token}</textarea>
			</label>
		{/if}
		<div class="buttons">
			{#if !token}
				<Button submit primary icon={loading ? loadingSpinner : undefined}>Create</Button>
			{:else}
				<Button secondary onclick={() => token && window.navigator.clipboard.writeText(token)}>
					Copy Token
				</Button>
			{/if}
			<Button secondary onclick={reset}>Close</Button>
		</div>
	</form>
</Dialog>

<style lang="scss">
	.layout {
		padding: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h1 {
		font-size: 1rem;
		font-weight: 600;
	}

	.token {
		min-height: 8rem;
	}

	.buttons {
		display: flex;
		gap: 1rem;
	}
</style>
