<script lang="ts">
	import Flags from "$/components/flags.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import moment from "moment/min/moment-with-locales";
	import { SealCheck } from "phosphor-svelte";
	import Date from "../date.svelte";
	import CountryFlag from "../country-flag.svelte";
	import { t } from "svelte-i18n";
	import { numberFormat } from "$/lib/utils";
	import EmoteTicketsTableActions from "./emote-tickets-table-actions.svelte";
	import EmoteTicketsTableActionsHeader from "./emote-tickets-table-actions-header.svelte";
	import { browser } from "$app/environment";
	import type { ButtonOptions } from "./emote-ticket.svelte";

	interface Props {
		buttonOptions: ButtonOptions;
		selectedMap: boolean[];
		onclick: (index: number) => void;
	}

	let { buttonOptions = $bindable(), selectedMap = $bindable(), onclick }: Props = $props();

	let allSelected = $derived(selectedMap.every((v) => v));
	let anySelected = $derived(selectedMap.some((v) => v));

	function selectAllClick() {
		selectedMap = Array(selectedMap.length).fill(!allSelected);
	}

	function loadActionsPosition(): "left" | "right" | null {
		if (!browser) return null;
		const position = window.localStorage.getItem("emoteTicketsActionsPosition");
		return position && JSON.parse(position);
	}

	let actionsPosition: "left" | "right" = $state(loadActionsPosition() || "left");

	$effect(() => {
		if (actionsPosition && browser) {
			window.localStorage.setItem("emoteTicketsActionsPosition", JSON.stringify(actionsPosition));
		}
	});
</script>

<table>
	<thead>
		<tr>
			<th class="shrink">
				<Checkbox
					value={allSelected}
					indeterminate={anySelected && !allSelected}
					onclick={selectAllClick}
				/>
			</th>
			{#if actionsPosition === "left"}
				<EmoteTicketsTableActionsHeader bind:buttonOptions bind:actionsPosition />
			{/if}
			<th>{$t("common.emotes", { values: { count: 1 } })}</th>
			<th>{$t("pages.admin.tickets.emote_table.uploader")}</th>
			<th>{$t("common.channels", { values: { count: 2 } })}</th>
			<th>{$t("pages.admin.tickets.emote_table.country")}</th>
			<th>{$t("pages.admin.tickets.emote_table.tags_flags")}</th>
			<th>{$t("pages.admin.tickets.emote_table.reviewed_by")}</th>
			<th>{$t("common.date")}</th>
			{#if actionsPosition === "right"}
				<EmoteTicketsTableActionsHeader bind:buttonOptions bind:actionsPosition />
			{/if}
		</tr>
	</thead>
	<tbody>
		{#each Array(selectedMap.length) as _, i}
			<tr class="data-row" onclick={() => onclick(i)}>
				<td class="shrink">
					<Checkbox bind:value={selectedMap[i]} />
				</td>
				{#if actionsPosition === "left"}
					<EmoteTicketsTableActions bind:buttonOptions />
				{/if}
				<td>
					<div class="emote">
						<!-- <EmotePreview emoteOnly style="pointer-events: none" /> -->
						EmoteName
					</div>
				</td>
				<td>
					<a href="/users/uploader" class="uploader">
						Username
						<SealCheck size={1 * 16} weight="fill" color="var(--store)" />
					</a>
				</td>
				<td>{numberFormat().format(999)}</td>
				<td>
					<CountryFlag code="gb" name="Great Britain" height={1.2 * 16} />
				</td>
				<td>
					<Flags flags={["overlaying", "lorem"]} />
				</td>
				<td>
					<div class="mod">
						<div class="placeholder"></div>
						Mod Name
					</div>
				</td>
				<td class="date shrink">
					<Date date={moment()} />
				</td>
				{#if actionsPosition === "right"}
					<EmoteTicketsTableActions bind:buttonOptions />
				{/if}
			</tr>
		{/each}
	</tbody>
</table>

<style lang="scss">
	.data-row {
		cursor: pointer;
	}

	.emote {
		display: grid;
		align-items: center;
		gap: 0.5rem;
		grid-template-columns: 2rem auto;
	}

	.uploader {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		color: var(--text);
	}

	.mod {
		display: flex;
		align-items: center;
		gap: 0.75rem;

		.placeholder {
			flex-shrink: 0;

			width: 2rem;
			height: 2rem;
			border-radius: 50%;
			background-color: var(--secondary);
		}
	}

	.date {
		color: var(--text-light);
		white-space: nowrap;
	}
</style>
