<script lang="ts">
	import Troll from "$/components/icons/troll.svelte";
	import { page } from "$app/stores";
	import { t } from "svelte-i18n";

	function defaultDetails(status: number): string {
		if (status === 401) {
			return "You are unauthorized to view this page.";
		} else if (status === 403) {
			return "You do not have permission to view this page.";
		} else if (status === 404) {
			return "The page you were looking for could not be found.";
		} else if (status === 500) {
			return "You can try refreshing the page. If the problem persists, please contact us.";
		} else {
			return "An error occurred.";
		}
	}

	let title = $derived($page.error?.message ?? "Error");
	let details = $derived($page.error?.details ?? defaultDetails($page.status));
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
