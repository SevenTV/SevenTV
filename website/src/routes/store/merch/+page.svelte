<script>
	import Banner from "$/components/store/banner.svelte";
</script>

<svelte:head>
	<title>Merch - 7TV</title>
</svelte:head>

<Banner title="Merch" subtitle="Coming soon!" gradientColor="#11bcff">
	<img
		src="/pausershype.webp"
		width="64"
		height="64"
		alt="Under construction"
	/>
</Banner>

<style lang="scss">
	img {
		transform: scaleX(-1);
	}
</style>
