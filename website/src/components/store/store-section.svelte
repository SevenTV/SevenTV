<script lang="ts">
    export let title: string | null = null;
</script>

<section>
    {#if title}
        <div class="header">
            <h2>{title}</h2>
            <slot name="header" />
        </div>
    {/if}
    <slot />
</section>

<style lang="scss">
    section {
        flex-basis: 0;
        flex-grow: 1;

        background-color: var(--bg-medium);
        border-radius: 0.5rem;
        padding: 1rem 1.25rem;

        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
    }

    h2 {
        font-size: 1rem;
        font-weight: 600;
    }
</style>
