<script lang="ts">
	import { priceFormat } from "$/lib/utils";
	import { Trash } from "phosphor-svelte";
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
        <table class="items">
            <thead>
                <tr>
                    <th>Preview</th>
                    <th>Item</th>
                    <th>Price</th>
                    <th></th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>
                        <div class="preview">
                            <div class="placeholder"></div>
                            <div class="placeholder"></div>
                            <div class="placeholder"></div>
                        </div>
                    </td>
                    <td class="name">Christmas Bundle</td>
                    <td class="price">{priceFormat.format(7.96)}</td>
                    <td class="actions">
                        <Button secondary>
                            <Trash slot="icon" />
                        </Button>
                    </td>
                </tr>
                <tr>
                    <td>
                        <div class="preview">
                            <div class="placeholder"></div>
                            <div class="placeholder"></div>
                            <div class="placeholder"></div>
                        </div>
                    </td>
                    <td class="name">Christmas Bundle</td>
                    <td class="price">{priceFormat.format(7.96)}</td>
                    <td class="actions">
                        <Button secondary>
                            <Trash slot="icon" />
                        </Button>
                    </td>
                </tr>
            </tbody>
        </table>
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

    .items {
        border-spacing: 0 0.5rem;
        border-collapse: separate;
        border: none;

        th {
            font-size: 0.75rem;
            color: var(--text-light);
            font-weight: 400;
            text-align: left;
        }

        td {
            padding: 0.5rem 0;
            background-color: var(--bg-light);

            &:first-child {
                padding-left: 0.5rem;
                border-radius: 0.5rem 0 0 0.5rem;
            }

            &:last-child {
                padding-right: 0.5rem;
                border-radius: 0 0.5rem 0.5rem 0;
            }
        }

        .preview {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .placeholder {
            width: 1.5rem;
            height: 1.5rem;
            background-color: var(--secondary);
            border-radius: 50%;
        }

        .name {
            font-size: 0.875rem;
            font-weight: 500;
        }

        .price {
            font-size: 0.875rem;
            font-weight: 600;
        }

        .actions {
            // Shrink column
            width: 1px;
        }
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
