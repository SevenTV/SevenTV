<script lang="ts">
	import type { Moment } from "moment/min/moment-with-locales";

	export let date: Moment;
</script>

<span title={date.fromNow()}>{date.format("ll")}</span>
