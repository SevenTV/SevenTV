<script lang="ts">
	import { PencilSimple, Trash } from "phosphor-svelte";
	import Button from "../button.svelte";
	import Checkbox from "../checkbox.svelte";
	import Date from "../date.svelte";
	import moment from "moment";
	import Flags from "../flags.svelte";

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
			<th>Name</th>
			<th>Last Modified</th>
			<th>Permissions</th>
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

<style lang="scss">
	table {
		width: 100%;
		border-collapse: collapse;

		background-color: var(--bg-light);
		border-radius: 0.5rem;
	}

	thead {
		text-align: left;
	}

	th {
		padding: 0.75rem;
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--text-light);

		border-bottom: 1px solid var(--border-active);
	}

	tr {
		&.data-row {
			cursor: pointer;
		}

		&:focus-visible,
		&:hover {
			td {
				background-color: var(--bg-medium);
			}
		}

		&:last-child > td {
			border-bottom: none;

			&:first-child {
				border-bottom-left-radius: 0.5rem;
			}

			&:last-child {
				border-bottom-right-radius: 0.5rem;
			}
		}
	}

	td {
		padding: 0.5rem 0.75rem;
		border-block: 1px solid var(--border-active);
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

	.shrink {
		// Shrink column width
		width: 0;
	}

	.buttons {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
</style>
