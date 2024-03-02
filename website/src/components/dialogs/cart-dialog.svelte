<script lang="ts">
	import { priceFormat } from "$/lib/utils";
    import Button from "../button.svelte";
    import Checkbox from "../checkbox.svelte";
	import SearchBar from "../search-bar.svelte";
    import Dialog from "./dialog.svelte";

    let gift = false;
</script>

<Dialog width={30} on:close>
    <div class="layout">
        <h1>Your Cart</h1>
        <hr />
        <Checkbox label="I want to gift someone" bind:value={gift} />
        {#if gift}
            <SearchBar grow />
        {/if}
        <div class="total">
            <span>Total</span>
            <span>{priceFormat.format(7.96)}</span>
        </div>
        <div class="buttons">
            <Button secondary>Cancel</Button>
            <Button primary>Proceed</Button>
        </div>
    </div>
</Dialog>

<style lang="scss">
    .layout {
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    h1 {
        font-size: 1rem;
        font-weight: 600;
        margin-block: 0.4rem;
    }

    .total {
        display: flex;
        justify-content: space-between;
        font-size: 1.25rem;
        font-weight: 600;

        margin-block: 0.5rem;
    }

    .buttons {
        display: flex;
        justify-content: flex-end;
        gap: 0.5rem;
    }
</style>
