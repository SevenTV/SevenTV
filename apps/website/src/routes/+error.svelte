<script lang="ts">
	import Error from "$/components/error.svelte";
	import { page } from "$app/stores";
	import { t } from "svelte-i18n";

	function defaultDetails(status: number): string {
		if (status === 401) {
			return $t("page_errors.unauthorized");
		} else if (status === 403) {
			return $t("page_errors.forbidden");
		} else if (status === 404) {
			return $t("page_errors.not_found");
		} else if (status === 500) {
			return $t("page_errors.unexpected");
		} else {
			return $t("page_errors.generic");
		} 
	}

	let title = $derived($page.error?.message ?? $t("dialogs.error.title"));
	let details = $derived($page.error?.details ?? defaultDetails($page.status));
</script>

<Error {title} {details} />
