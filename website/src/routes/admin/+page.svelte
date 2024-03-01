<script lang="ts">
	import MonthlyPaints from "$/components/store/monthly-paints.svelte";
</script>

<svelte:head>
    <title>Overview - 7TV Admin</title>
</svelte:head>

<div class="grid">
    <MonthlyPaints />
</div>

<style lang="scss">
    .grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(30rem, 1fr));
        gap: 1rem;
    }
</style>
