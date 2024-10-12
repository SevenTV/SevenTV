<script lang="ts">
	import { t } from "svelte-i18n";
	import Connections from "$/components/profile/connections.svelte";
	import type { PageData } from "./$types";

	export let data: PageData;
</script>

<div class="layout">
	<h2>{$t("common.connections")}</h2>
	<div class="link-list">
		<Connections user={data.user} big />
	</div>
	<h2>{$t("common.editors")}</h2>
	<div class="link-list">
		<!-- <ChannelPreview big />
		<ChannelPreview big /> -->
	</div>
</div>

<style lang="scss">
	.layout {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	h2 {
		font-size: 1rem;
		font-weight: 600;
	}

	.link-list {
		display: flex;
		flex-direction: column;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}
</style>
