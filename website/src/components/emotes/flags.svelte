<script lang="ts">
	import { Eye, Fire, GlobeSimple, SealCheck, StackSimple } from "phosphor-svelte";

    const icons: {[key: string]: any} = {
        "global": GlobeSimple,
        "trending": Fire,
        "overlay": StackSimple,
        "verified": SealCheck,
        "public": Eye,
    };

    export let flags: string[] = [];
</script>

<div class="flags" {...$$restProps}>
    {#each flags as flag}
        <span class="flag">
            <svelte:component this={icons[flag]} size="1rem" />
            <span class="hide-on-mobile">{flag}</span>
        </span>
    {/each}
</div>

<style lang="scss">
    .flags {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

    .flag {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.3rem 0.5rem 0.3rem 0.62rem;
        background-color: var(--secondary);
        border-radius: 0.5rem;

        color: var(--text-light);
        font-size: 0.75rem;
        font-weight: 500;
        text-transform: capitalize;
    }

    @media screen and (max-width: 960px) {
	    .flag {
			padding: 0.3rem 0.5rem;
		}
	}
</style>
