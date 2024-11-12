<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { MagnifyingGlass, PencilSimple, Trash, UserCirclePlus } from "phosphor-svelte";
	import UserTable from "./user-table.svelte";
	import HideOn from "../hide-on.svelte";
	import { t } from "svelte-i18n";

	let { showAddEditor = false }: { showAddEditor?: boolean } = $props();

	let selectedMap: boolean[] = $state(Array(20).fill(false));
	let selectMode = $derived(selectedMap.some((v) => v));
</script>

<nav class="nav-bar">
	<div class="buttons">
		<div class="link-list">
			<TabLink title={$t("common.editors")} href="/settings/editors" />
			<TabLink
				title={$t("pages.settings.editors.editing_for")}
				href="/settings/editors/editing-for"
			/>
		</div>
		{#if showAddEditor}
			<TextInput placeholder={$t("pages.settings.editors.add_editor")}>
				{#snippet icon()}
					<UserCirclePlus />
				{/snippet}
			</TextInput>
		{/if}
		{#if selectMode}
			<Button style="border: none">
				{#snippet icon()}
					<PencilSimple />
				{/snippet}
			</Button>
			<Button style="border: none">
				{#snippet icon()}
					<Trash />
				{/snippet}
			</Button>
		{/if}
	</div>
	<HideOn mobile>
		<TextInput placeholder={$t("labels.search")}>
			{#snippet icon()}
				<MagnifyingGlass />
			{/snippet}
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
