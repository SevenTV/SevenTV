<script lang="ts">
	import UserTable from "$/components/settings/user-table.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import Button from "$/components/input/button.svelte";
	import { MagnifyingGlass, Prohibit, PencilSimple, Trash } from "phosphor-svelte";
	import { t } from "svelte-i18n";

	let selectedMap = Array(10).fill(false);

	$: selectMode = selectedMap.some((v) => v);
</script>

<svelte:head>
	<title>{$t("page_titles.blocked_settings")} - 7TV</title>
</svelte:head>

<section>
	<div>
		<h2>{$t("pages.settings.blocked.title")}</h2>
		<span class="details">{$t("pages.settings.blocked.details")}</span>
	</div>
	<div class="content">
		<nav class="nav-bar">
			<div class="buttons">
				<TextInput placeholder={$t("pages.settings.blocked.add_user")}>
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
			<TextInput placeholder={$t("labels.search")}>
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
