<script lang="ts">
	import Button from "$/components/button.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { MagnifyingGlass, PencilSimple, Trash, UserCirclePlus } from "phosphor-svelte";
	import UserTable from "./user-table.svelte";
	import HideOn from "../hide-on.svelte";

	export let showAddEditor: boolean = false;

	let selectedMap: boolean[] = Array(20).fill(false);
	$: selectMode = selectedMap.some((v) => v);
</script>

<nav class="nav-bar">
	<div class="buttons">
		<div class="link-list">
			<TabLink title="Editors" href="/settings/editors" />
			<TabLink title="Editing For" href="/settings/editors/editing-for" />
		</div>
		{#if showAddEditor}
			<TextInput placeholder="Add Editor">
				<UserCirclePlus slot="icon" />
			</TextInput>
		{/if}
		{#if selectMode}
			<Button style="border: none">
				<PencilSimple slot="icon" />
			</Button>
			<Button style="border: none">
				<Trash slot="icon" />
			</Button>
		{/if}
	</div>
	<HideOn mobile>
		<TextInput placeholder="Search">
			<MagnifyingGlass slot="icon" />
		</TextInput>
	</HideOn>
</nav>
<UserTable bind:selectedMap />

<style lang="scss">
	.nav-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;

		.buttons {
			display: flex;
			align-items: center;
			gap: 0.5rem;
		}
	}

	.link-list {
		display: flex;
		background-color: var(--bg-light);
		border-radius: 0.5rem;
	}
</style>
