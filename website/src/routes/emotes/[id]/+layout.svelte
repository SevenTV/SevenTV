<script lang="ts">
	import type { LayoutData } from "./$types";
	import { t } from "svelte-i18n";
	import EmoteInfo from "$/components/emotes/emote-info.svelte";

	export let data: LayoutData;
</script>

<svelte:head>
	<title>{data.id} - {$t("page_titles.suffix")}</title>
</svelte:head>

<div class="layout">
	<div>
		<EmoteInfo id={data.id} />
	</div>
	<div class="tabs">
		<slot />
	</div>
</div>

<style lang="scss">
	.layout {
		width: 100%;
		max-width: 80rem;
		margin-inline: auto;

		padding: 1.25rem;
		min-height: 100%;

		display: flex;
		flex-direction: column;
		gap: 1rem;

		& > * {
			background-color: var(--bg-medium);
			border: 1px solid var(--layout-border);
			border-radius: 0.5rem;
			padding: 1rem;
		}
	}

	.tabs {
		flex-grow: 1;
	}

	@media screen and (max-width: 960px) {
		.layout {
			padding: 0.5rem;
		}
	}
</style>
