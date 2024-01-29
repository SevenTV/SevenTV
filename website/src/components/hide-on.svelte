<script lang="ts">
	export let mobile = false;
	export let desktop = false;
</script>

<div class:hide-on-mobile={mobile} class:hide-on-desktop={desktop}>
	<slot />
</div>

<style lang="scss">
	div {
		display: contents;
	}
</style>
