<script lang="ts">
	import TabLink from "$/components/tab-link.svelte";
	import { Flag, PaintBrush, Table } from "phosphor-svelte";
</script>

<div class="side-bar-layout">
	<aside class="side-bar">
		<h1>Admin Dashboard</h1>
		<nav class="link-list">
			<TabLink title="Overview" href="/admin">
				<Table />
				<Table weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Reports" href="/admin/reports">
				<Flag />
				<Flag weight="fill" slot="active" />
			</TabLink>
			<TabLink title="Cosmetics" href="/admin/cosmetics">
				<PaintBrush />
				<PaintBrush weight="fill" slot="active" />
			</TabLink>
		</nav>
		<img src="/modge.webp" width="64" height="64" alt="Modge" class="modge" />
	</aside>
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.modge {
		margin-left: auto;
		margin-top: auto;
		filter: saturate(0) opacity(0.2);
	}
</style>
