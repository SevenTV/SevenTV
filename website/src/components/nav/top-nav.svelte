<script lang="ts">
    import Logo from "$/components/icons/logo.svelte";
    import Tabs from "./tabs.svelte";
</script>

<nav>
    <div class="links">
        <a class="home" href="/">
            <Logo />
        </a>
        <Tabs
            tabs={[
                { name: "Emotes", pathname: "/emotes" },
                { name: "Discover", pathname: "/discover" },
                { name: "Store", pathname: "/store" },
                { name: "More", pathname: "/more" },
            ]}
        />
    </div>
    <search>Search</search>
    <div>User</div>
</nav>

<style lang="scss">
    nav {
        background-color: var(--bg-dark);
        border-bottom: 1px solid var(--border);
        padding: 0 2rem;

        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 2rem;
    }

    .links {
        display: flex;
        gap: 2rem;

        .home {
            display: flex;
            align-items: center;
        }
    }
</style>
