<script lang="ts">
	import UserTable from "$/components/settings/user-table.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import Button from "$/components/button.svelte";
	import { MagnifyingGlass, Prohibit, PencilSimple, Trash } from "phosphor-svelte";

	let selectedMap = Array(10).fill(false);

	$: selectMode = selectedMap.some((v) => v);
</script>

<svelte:head>
	<title>Blocked Settings - 7TV</title>
</svelte:head>

<section>
	<div>
		<h2>Blocked</h2>
		<span class="details"
			>Display a list of restricted or prohibited content, users, or activities</span
		>
	</div>
	<div class="content">
		<nav class="nav-bar">
			<div class="buttons">
				<TextInput placeholder="Add Blocked User">
					<Prohibit slot="icon" />
				</TextInput>
				{#if selectMode}
					<Button style="border: none">
						<PencilSimple slot="icon" />
					</Button>
					<Button style="border: none">
						<Trash slot="icon" />
					</Button>
				{/if}
			</div>
			<TextInput placeholder="Search">
				<MagnifyingGlass slot="icon" />
			</TextInput>
		</nav>
		<UserTable bind:selectedMap />
	</div>
</section>

<style lang="scss">
	@import "../../../styles/settings.scss";

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
</style>
