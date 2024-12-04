<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { numberFormat } from "$/lib/utils";
	import { page } from "$app/stores";
	import { CaretDown, CaretUp, Flag, PaintBrush, Smiley, Table, Ticket } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { type Page } from "@sveltejs/kit";
	import modge from "$assets/modge.webp?url";

	let { children } = $props();

	let ticketsSelected = $derived($page.url.pathname.startsWith("/admin/tickets"));

	function customMatcher(page: Page, href: string | undefined) {
		return !!href && page.url.pathname.startsWith(href);
	}
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>{$t("pages.admin.title")}</h1>
		<nav class="link-list">
			<TabLink title={$t("pages.admin.overview")} href="/admin" big>
				<Table />
				{#snippet active()}
					<Ticket weight="fill" />
				{/snippet}
			</TabLink>
			<TabLink
				title={$t("pages.admin.tickets.title")}
				href="/admin/tickets"
				matcher={customMatcher}
				big
			>
				<Ticket />
				{#snippet active()}
					<Ticket weight="fill" />
				{/snippet}
				{#snippet iconRight()}
					{#if ticketsSelected}
						<CaretUp />
					{:else}
						<CaretDown />
					{/if}
				{/snippet}
			</TabLink>
			{#if ticketsSelected}
				<div class="indent link-list">
					<TabLink
						title="{$t('common.emotes', { values: { count: 2 } })} ({numberFormat().format(1920)})"
						href="/admin/tickets/emotes"
						matcher={customMatcher}
					>
						<Smiley />
						{#snippet active()}
							<Smiley weight="fill" />
						{/snippet}
					</TabLink>
					<TabLink
						title="{$t('pages.admin.tickets.reports')} ({numberFormat().format(2)})"
						href="/admin/tickets/reports"
					>
						<Flag />
						{#snippet active()}
							<Smiley weight="fill" />
							<Flag weight="fill" />
						{/snippet}
					</TabLink>
				</div>
			{/if}
			<TabLink title={$t("common.cosmetics")} href="/admin/cosmetics" big>
				<PaintBrush />
				{#snippet active()}
					<PaintBrush weight="fill" />
				{/snippet}
			</TabLink>
		</nav>
		<img src={modge} width="64" height="64" alt="Modge" class="modge hide-on-mobile" />
	</aside>
	<div class="content">
		{@render children()}
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
