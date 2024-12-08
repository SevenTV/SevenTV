<script lang="ts">
	import { createSet } from "$/lib/setMutations";
	import { goto } from "$app/navigation";
	import Button from "../input/button.svelte";
	import TagsInput from "../input/tags-input.svelte";
	import TextInput from "../input/text-input.svelte";
	import Spinner from "../spinner.svelte";
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import { t } from "svelte-i18n";

	interface Props {
		mode: DialogMode;
	}

	let { mode = $bindable("hidden") }: Props = $props();

	let name = $state("");
	let tags = $state([]);
	let loading = $state(false);

	async function submit() {
		loading = true;

		const set = await createSet(name, tags);

		loading = false;
		mode = "hidden";

		if (set) {
			goto(`/emote-sets/${set.id}`);
		}
	}
</script>

<Dialog width={30} bind:mode>
	<form class="layout">
		<h1>Create Emote Set</h1>
		<TextInput placeholder="Name" bind:value={name}>
			<span class="label">Emote Set Name</span>
		</TextInput>
		<TagsInput bind:tags />
		<div class="buttons">
			{#snippet loadingSpinner()}
				<Spinner />
			{/snippet}
			<Button secondary onclick={() => (mode = "hidden")}>{$t("labels.cancel")}</Button>
			<Button
				submit
				primary
				disabled={loading || !name}
				onclick={submit}
				icon={loading ? loadingSpinner : undefined}
			>
				Create
			</Button>
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

	.buttons {
		grid-column: span 2;

		display: flex;
		gap: 0.5rem;
		justify-content: flex-end;
	}
</style>
