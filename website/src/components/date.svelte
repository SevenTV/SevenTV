<script lang="ts">
	import type { Moment } from "moment";

	export let date: Moment;
</script>

<span title={date.fromNow()}>{date.format("ll")}</span>
