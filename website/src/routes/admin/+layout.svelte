<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { numberFormat } from "$/lib/utils";
	import { page } from "$app/stores";
	import { CaretDown, CaretUp, Flag, PaintBrush, Smiley, Table, Ticket } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	$: ticketsSelected = $page.url.pathname.startsWith("/admin/tickets");

	function customMatcher(_id: string | null, url: URL, href: string | null) {
		return !!href && url.pathname.startsWith(href);
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.admin.title")}</h1>
		<nav class="link-list">
			<TabLink title={$t("pages.admin.overview")} href="/admin" big>
				<Table />
				<Table weight="fill" slot="active" />
			</TabLink>
			<TabLink title={$t("pages.admin.tickets.title")} href="/admin/tickets" matcher={customMatcher} big>
				<Ticket />
				<Ticket weight="fill" slot="active" />
				<svelte:fragment slot="icon-right">
					{#if ticketsSelected}
						<CaretUp />
					{:else}
						<CaretDown />
					{/if}
				</svelte:fragment>
			</TabLink>
			{#if ticketsSelected}
				<div class="indent link-list">
					<TabLink title="{$t("common.emotes", { values: { count: 2 } })} ({numberFormat().format(1920)})" href="/admin/tickets/emotes" matcher={customMatcher}>
						<Smiley />
						<Smiley weight="fill" slot="active" />
					</TabLink>
					<TabLink title="{$t("pages.admin.tickets.reports")} ({numberFormat().format(2)})" href="/admin/tickets/reports">
						<Flag />
						<Flag weight="fill" slot="active" />
					</TabLink>
				</div>
			{/if}
			<TabLink title={$t("common.cosmetics")} href="/admin/cosmetics" big>
				<PaintBrush />
				<PaintBrush weight="fill" slot="active" />
			</TabLink>
		</nav>
		<img src="/modge.webp" width="64" height="64" alt="Modge" class="modge hide-on-mobile" />
	</aside>
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.indent {
		margin-left: 1rem;
	}

	.modge {
		margin-left: auto;
		margin-top: auto;
		filter: saturate(0) opacity(0.2);
	}

	.content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
</style>
