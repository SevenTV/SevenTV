<script lang="ts">
	import type { Moment } from "moment/min/moment-with-locales";

	let { date, showExact = false }: { date: Moment; showExact?: boolean } = $props();
</script>

<span title={date.format("lll")}>{date.fromNow()}{showExact ? ` (${date.format("ll")})` : ""}</span>
