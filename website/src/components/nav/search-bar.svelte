<script lang="ts">
    import Fa from "svelte-fa";
    import { faSearch } from "@fortawesome/free-solid-svg-icons";
</script>

<search>
    <div class="input-wrapper">
        <div class="icon">
            <Fa icon={faSearch} />
        </div>
        <input type="text" placeholder="Search" />
    </div>
    <select>
        <option value="emotes">Emotes</option>
        <option value="users">Users</option>
    </select>
</search>

<style lang="scss">
    search {
        flex-grow: 1;
        max-width: 30rem;

        display: flex;
        align-items: center;
        justify-content: center;

        .input-wrapper {
            flex-grow: 1;
            position: relative;
        }

        .icon {
            position: absolute;
            top: 0;
            left: 0;
            bottom: 0;

            display: flex;
            align-items: center;
            padding-left: 1rem;
            color: var(--text-light);
            pointer-events: none;
        }

        input {
            width: calc(100% - 2.5rem);

            padding: 0.75rem 0;
            padding-left: 2.5rem;
            border-radius: 0.5rem 0 0 0.5rem;
            border: none;
            border-right: 1px solid var(--border);
            outline: none;
            background-color: var(--bg-lightest);
            color: var(--text);
            font-size: 0.875rem;
        }

        select {
            padding: 0.75rem 1rem;
            border-radius: 0 0.5rem 0.5rem 0;
            border: none;
            outline: none;
            background-color: var(--bg-lightest);
            color: var(--text);
            font-size: 0.875rem;
            font-weight: 500;
        }
    }
</style>
