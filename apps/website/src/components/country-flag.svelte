<script lang="ts">
	export let code: string;
	export let name: string;
</script>

<img src="/flags/{code}.svg" alt={name} title={name} class="flag" {...$$restProps} />

<style lang="scss">
	.flag {
		border-radius: 0.25rem;
	}
</style>
