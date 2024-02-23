<script lang="ts">
	import { page } from "$app/stores";
	import { fly } from "svelte/transition";
	import { sideBar } from "$/lib/stores";
	import { ArrowLineLeft, ArrowLineRight, PaintBrush, Star, TShirt } from "phosphor-svelte";
</script>

<div class="side-bar-layout">
	{#if $sideBar}
		<aside class="side-bar" transition:fly={{ x: -16 * 16, duration: 200, opacity: 1 }}>
			<button class="button square collapse" on:click={() => ($sideBar = false)}>
				<ArrowLineLeft />
			</button>
			<h1>Store</h1>
			<div class="link-list">
				<a class="button big" href="/store" class:secondary={$page.url.pathname === "/store"}>
					<Star />
					Subscription
				</a>
				<a
					class="button big"
					href="/store/paint-bundles"
					class:secondary={$page.url.pathname === "/store/paint-bundles"}
				>
					<PaintBrush />
					Paint Bundles
				</a>
				<a
					class="button big"
					href="/store/merch"
					class:secondary={$page.url.pathname === "/store/merch"}
				>
					<TShirt />
					Merch
				</a>
			</div>
			<hr />
			<label class="redeem">
				Redeem Gift Code
				<input type="text" placeholder="Enter code" />
			</label>
		</aside>
	{:else}
		<button class="button square expand" on:click={() => ($sideBar = true)}>
			<ArrowLineRight />
		</button>
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

		.button.collapse {
			position: absolute;
			top: 1rem;
			right: 1rem;
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

	.expand {
		position: absolute;
		top: 1rem;
		left: 1rem;

		z-index: 2;
	}
</style>
