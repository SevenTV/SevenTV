<script lang="ts">
	import Troll from "$/components/icons/troll.svelte";
	import { t } from "svelte-i18n";

	let { title, details } = $props();
</script>

<svelte:head>
	<title>{title} - {$t("page_titles.suffix")}</title>
</svelte:head>

<div class="container">
	<div class="troll">
		<Troll />
	</div>
	<h1>{title}</h1>
	<p>{details}</p>
	<a href="/">Go Back</a>
</div>

<style lang="scss">
	.container {
		height: 100%;

		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		text-align: center;

		font-size: 1.25rem;
	}

	.troll {
		font-size: 12rem;
	}

	a {
		color: var(--text);
	}
</style>
