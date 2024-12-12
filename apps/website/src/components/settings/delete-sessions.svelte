<script lang="ts">
	import { deleteAllSessions } from "$/lib/userMutations";
	import Button from "../input/button.svelte";
	import Spinner from "../spinner.svelte";
	import { t } from "svelte-i18n";
	import { user } from "$/lib/auth";
	import { Check } from "phosphor-svelte";

	let state = $state<"idle" | "loading" | "done">("idle");

	async function deleteSessions() {
		state = "loading";
		await deleteAllSessions($user!.id);
		state = "done";
	}
</script>

{#snippet loadingSpinner()}
	<Spinner />
{/snippet}
{#snippet doneCheck()}
	<Check />
{/snippet}

<Button
	secondary
	style="align-self: flex-start"
	icon={state === "loading" ? loadingSpinner : state === "done" ? doneCheck : undefined}
	disabled={state !== "idle"}
	onclick={deleteSessions}
>
	{$t("pages.settings.account.security.sign_out_everywhere")}
</Button>
