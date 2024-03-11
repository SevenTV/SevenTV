<script lang="ts">
	import { CaretRight } from "phosphor-svelte";

	export let href: string | null = null;
    export let showCaret: boolean = false;

	export let hideOnMobile: boolean = false;
	export let hideOnDesktop: boolean = false;
</script>

{#if href}
	<a
		{href}
		on:click
		class:hide-on-mobile={hideOnMobile}
		class:hide-on-desktop={hideOnDesktop}
		{...$$restProps}
	>
		<slot />
        {#if showCaret}
            <div class="caret">
                <CaretRight />
            </div>
        {/if}
	</a>
{:else}
	<button
		type="button"
		on:click
		class:hide-on-mobile={hideOnMobile}
		class:hide-on-desktop={hideOnDesktop}
		{...$$restProps}
	>
		<slot />
        {#if showCaret}
            <div class="caret">
                <CaretRight />
            </div>
        {/if}
	</button>
{/if}

<style lang="scss">
	a,
	button {
		padding: 0.75rem 1.2rem;
		border-radius: 0.5rem;
		color: var(--text);
		font-size: 0.875rem;
		font-weight: 500;
		text-decoration: none;

		display: flex;
		align-items: center;
		gap: 1.2rem;

		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}
	}

	.caret {
		flex-grow: 1;
		justify-self: end;
		text-align: right;
	}

    @media screen and (max-width: 960px) {
        a,
        button {
            padding: 1rem;
            font-size: 1rem;
            gap: 0.75rem;
        }
	}
</style>
