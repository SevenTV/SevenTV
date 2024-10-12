<script lang="ts">
	export let mobile = false;
	export let desktop = false;
</script>

<div class="hide-on" class:hide-on-mobile={mobile} class:hide-on-desktop={desktop}>
	<slot />
</div>

<style lang="scss">
	.hide-on {
		display: contents;
	}
</style>
