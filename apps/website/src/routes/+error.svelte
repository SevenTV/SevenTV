<script lang="ts">
	import Error from "$/components/error.svelte";
	import { page } from "$app/stores";

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

<Error {title} {details} />
