<script lang="ts">
	import EmotePreview from "$/components/emote-preview.svelte";
	import Flags from "$/components/flags.svelte";
	import Button from "$/components/input/button.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import moment from "moment/min/moment-with-locales";
	import { ArrowsMerge, Check, EyeSlash, Gear, SealCheck, Trash } from "phosphor-svelte";
	import Date from "../date.svelte";
	import CountryFlag from "../country-flag.svelte";
	import { t } from "svelte-i18n";
	import { numberFormat } from "$/lib/utils";
	import { createEventDispatcher } from "svelte";

	const dispatch = createEventDispatcher();

	export let selectedMap: boolean[];

	$: allSelected = selectedMap.every((v) => v);
	$: anySelected = selectedMap.some((v) => v);

	function selectAllClick() {
		selectedMap = Array(selectedMap.length).fill(!allSelected);
	}
</script>

<table>
	<thead>
		<tr>
			<th class="shrink">
				<Checkbox
					value={allSelected}
					indeterminate={anySelected && !allSelected}
					on:click={selectAllClick}
				/>
			</th>
			<th class="shrink">
				<Button>
					<Gear slot="icon" />
				</Button>
			</th>
			<th>{$t("common.emotes", { values: { count: 1 } })}</th>
			<th>{$t("pages.admin.tickets.emote_table.uploader")}</th>
			<th>{$t("common.channels", { values: { count: 2 } })}</th>
			<th>{$t("pages.admin.tickets.emote_table.country")}</th>
			<th>{$t("pages.admin.tickets.emote_table.tags_flags")}</th>
			<th>{$t("pages.admin.tickets.emote_table.reviewed_by")}</th>
			<th>{$t("common.date")}</th>
		</tr>
	</thead>
	<tbody>
		{#each Array(selectedMap.length) as _, i}
			<tr class="data-row" on:click={() => dispatch("click", i)}>
				<td class="shrink">
					<Checkbox bind:value={selectedMap[i]} />
				</td>
				<td class="shrink">
					<div class="buttons">
						<Button>
							<ArrowsMerge
								slot="icon"
								style="transform: rotate(-90deg)"
								color="var(--admin-merge)"
							/>
						</Button>
						<Button>
							<Trash slot="icon" color="var(--danger)" />
						</Button>
						<Button>
							<EyeSlash slot="icon" color="var(--admin-unlist)" />
						</Button>
						<Button>
							<Check slot="icon" color="var(--admin-approve)" />
						</Button>
					</div>
				</td>
				<td>
					<div class="emote">
						<EmotePreview emoteOnly style="pointer-events: none" />
						EmoteName
					</div>
				</td>
				<td>
					<a href="/user/uploader" class="uploader">
						Username
						<SealCheck size="1rem" weight="fill" color="var(--subscriber)" />
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
			</tr>
		{/each}
	</tbody>
</table>

<style lang="scss">
	.buttons {
		display: flex;
		align-items: center;
	}

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
