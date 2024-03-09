<script lang="ts">
	import { CaretRight } from "phosphor-svelte";
    import PaymentBrand from "../icons/payment-brand.svelte";
	import Button from "../button.svelte";

    export let type: string;
</script>

<label class="method">
    <input type="checkbox" />
    <div class="header">
        <PaymentBrand {type} />
        <div class="caret">
            <CaretRight />
        </div>
    </div>
    <div class="info">
        {#if type === "paypal"}
            <p>
                <span class="heading">Paypal Account</span>
                <br />
                <span class="details">ayyybubu@gmail.com</span>
            </p>
        {:else}
            <p>
                <span class="heading">Card Info</span>
                <br />
                <span class="details">
                    Ends in 2215
                    <br />
                    Expires 06/2029
                </span>
            </p>
            <p style="grid-row: span 2">
                <span class="heading">Address</span>
                <br />
                <span class="details">
                    Sebastian Fors
                    <br />
                    655 Rossville Ave
                    <br />
                    North Dakota
                    <br />
                    10309
                    <br />
                    (718) 984-7500
                </span>
            </p>
            <p>
                <span class="heading">Name</span>
                <br />
                <span class="details">Sebastian Fors</span>
            </p>
        {/if}
        <div class="buttons">
            <Button style="color: var(--error)">Remove</Button>
            <Button secondary>Edit</Button>
        </div>
    </div>
</label>

<style lang="scss">
    .method {
        padding: 1rem;
        background-color: var(--bg-light);
        border-radius: 0.5rem;
    }

    input {
        position: absolute;
        margin: 0;
        width: 0;
        height: 0;
        display: inline;
        -webkit-appearance: none;
        -moz-appearance: none;
        appearance: none;
        outline: none;

        &:checked {
            & ~ .info {
                display: grid;
            }
            
            & ~ .header .caret {
                transform: rotate(90deg);
            }
        }
    }

    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;

        font-size: 0.875rem;
        font-weight: 500;

        .caret {
            transition: transform 0.2s;
        }
    }

    .info {
        margin-top: 1rem;
        display: grid;
        grid-template-columns: repeat(2, auto);
        gap: 1rem;
        text-align: left;

        display: none;
    }

    .heading {
        font-size: 0.875rem;
        font-weight: 500;
    }

    .details {
        color: var(--text-light);
        font-size: 0.875rem;
    }

    .buttons {
        grid-column: 1 / -1;

        display: flex;
        justify-content: flex-end;
        align-items: center;
        gap: 0.5rem;
    }
</style>
