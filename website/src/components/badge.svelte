<script lang="ts">
    export let count: number;

    $: text = count > 99 ? "99+" : count.toString();
</script>

<div class="badge-container">
    <span class="badge">{text}</span>
    <slot />
</div>

<style lang="scss">
    .badge-container {
        position: relative;
        display: inline-block;
    }

    .badge {
        position: absolute;
        top: -0.55rem;
        right: -0.75rem;

        display: flex;
        align-items: center;
        justify-content: center;

        width: 1rem;
        height: 1rem;
        border-radius: 50%;
        background-color: var(--error);
        border: 3px solid var(--bg-dark);
        font-size: 0.625rem;
        font-weight: 500;
    }
</style>
