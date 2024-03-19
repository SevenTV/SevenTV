<script lang="ts">
	import { PencilSimple, Trash } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import Checkbox from "../input/checkbox.svelte";
	import Date from "../date.svelte";
	import moment from "moment/min/moment-with-locales";
	import Flags from "../flags.svelte";
	import { t } from "svelte-i18n";

	export let selectedMap: boolean[];

	$: allSelected = selectedMap.every((v) => v);
	$: anySelected = selectedMap.some((v) => v);

	function selectAllClick() {
		selectedMap = Array(selectedMap.length).fill(!allSelected);
	}

	function buttonClick(e: MouseEvent) {
		e.stopPropagation();
	}
</script>

<div class="scroll">
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
				<th>{$t("pages.settings.user_table.name")}</th>
				<th>{$t("pages.settings.user_table.last_modified")}</th>
				<th>{$t("pages.settings.user_table.permissions")}</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			{#each Array(selectedMap.length) as _, i}
				<tr class="data-row" on:click={() => (selectedMap[i] = !selectedMap[i])}>
					<td class="shrink">
						<Checkbox bind:value={selectedMap[i]} style="pointer-events: none" />
					</td>
					<td>
						<div class="user-info">
							<div class="placeholder"></div>
							<span class="name">user{i}</span>
						</div>
					</td>
					<td class="date">
						<Date date={moment("2024-01-22")} />
					</td>
					<td>
						<Flags
							flags={["profile", "editors", "emote_sets", "emotes"]}
							add={(e) => e.stopPropagation()}
						/>
					</td>
					<td class="shrink">
						<div class="buttons">
							<Button on:click={buttonClick}>
								<PencilSimple slot="icon" />
							</Button>
							<Button on:click={buttonClick}>
								<Trash slot="icon" />
							</Button>
						</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style lang="scss">
	.scroll {
		overflow: auto;
		overflow: overlay;
		scrollbar-gutter: stable;
	}

	.data-row {
		cursor: pointer;
	}

	.user-info {
		display: flex;
		align-items: center;
		gap: 1rem;

		.placeholder {
			width: 2.5rem;
			height: 2.5rem;
			background-color: var(--secondary);
			border-radius: 50%;
		}

		.name {
			font-size: 0.875rem;
			font-weight: 500;
		}
	}

	.date {
		color: var(--text-light);
		font-size: 0.875rem;
	}

	.buttons {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
