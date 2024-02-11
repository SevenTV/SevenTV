<script lang="ts">
	import { page } from "$app/stores";
	import {
		faArrowLeftFromLine,
		faArrowRightToLine,
		faSprayCan,
		faTshirt,
	} from "@fortawesome/pro-regular-svg-icons";
	import { faStar } from "@fortawesome/pro-solid-svg-icons";
	import Fa from "svelte-fa";
	import { fly } from "svelte/transition";
	import { sideBar } from "$/lib/stores";
</script>

<div class="side-bar-layout">
	{#if $sideBar}
		<div class="side-bar" transition:fly={{ x: -16 * 16, duration: 200, opacity: 1 }}>
			<button class="button collapse" on:click={() => ($sideBar = false)}>
				<Fa icon={faArrowLeftFromLine} size="1.2x" fw />
			</button>
			<h1>Store</h1>
			<div class="link-list">
				<a class="button big" href="/store" class:secondary={$page.url.pathname === "/store"}>
					<Fa icon={faStar} size="1.2x" fw />
					Subscription
				</a>
				<a
					class="button big"
					href="/store/paint-bundles"
					class:secondary={$page.url.pathname === "/store/paint-bundles"}
				>
					<Fa icon={faSprayCan} size="1.2x" fw />
					Paint Bundles
				</a>
				<a
					class="button big"
					href="/store/merch"
					class:secondary={$page.url.pathname === "/store/merch"}
				>
					<Fa icon={faTshirt} size="1.2x" fw />
					Merch
				</a>
			</div>
			<hr />
			<label class="redeem">
				Redeem Gift Code
				<input type="text" placeholder="Enter code" />
			</label>
		</div>
    {:else}
        <button class="button expand" on:click={() => ($sideBar = true)}>
            <Fa icon={faArrowRightToLine} size="1.2x" fw />
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
    }
</style>
