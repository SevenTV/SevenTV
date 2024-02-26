<script lang="ts">
	import { page } from "$app/stores";
	import { fly } from "svelte/transition";
	import { sideBar } from "$/lib/stores";
	import { ArrowLineLeft, ArrowLineRight, PaintBrush, Star, TShirt } from "phosphor-svelte";
	import Button from "$/components/button.svelte";
</script>

<div class="side-bar-layout">
	{#if $sideBar}
		<aside class="side-bar" transition:fly={{ x: -16 * 16, duration: 200, opacity: 1 }}>
			<Button
				on:click={() => ($sideBar = false)}
				style="position: absolute; top: 1rem; right: 1rem;"
			>
				<ArrowLineLeft slot="icon" />
			</Button>
			<h1>Store</h1>
			<div class="link-list">
				<Button big href="/store" primary={$page.url.pathname === "/store"}>
					<Star slot="icon" />
					Subscription
				</Button>
				<Button
					big
					href="/store/paint-bundles"
					primary={$page.url.pathname === "/store/paint-bundles"}
				>
					<PaintBrush slot="icon" />
					Paint Bundles
				</Button>
				<Button big href="/store/merch" primary={$page.url.pathname === "/store/merch"}>
					<TShirt slot="icon" />
					Merch
				</Button>
			</div>
			<hr />
			<label class="redeem">
				Redeem Gift Code
				<input type="text" placeholder="Enter code" />
			</label>
		</aside>
	{:else}
		<Button
			on:click={() => ($sideBar = true)}
			style="position: absolute; top: 1rem; left: 1rem; z-index: 2;"
		>
			<ArrowLineRight slot="icon" />
		</Button>
	{/if}
	<div class="content">
		<slot />
	</div>
</div>

<style lang="scss">
	.side-bar-layout {
		position: relative;
	}

	.side-bar {
		z-index: 1;

		h1 {
			font-size: 1.125rem;
			font-weight: 600;
			margin: 0.25rem 0;
		}

		.link-list {
			display: flex;
			flex-direction: column;
			gap: 0.25rem;
		}

		.redeem {
			display: flex;
			flex-direction: column;
			gap: 0.5rem;

			color: var(--text-lighter);
			font-size: 0.75rem;
			font-weight: 500;
		}
	}
</style>
