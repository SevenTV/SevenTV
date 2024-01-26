<script lang="ts">
    import Logo from "$/components/icons/logo.svelte";
    import SearchBar from "$/components/nav/search-bar.svelte";
    import Tabs from "./tabs.svelte";

    import Fa from "svelte-fa";
    import { faBell, faMessage, faPlusSquare } from "@fortawesome/free-regular-svg-icons";
    import { faPlus, faSearch } from "@fortawesome/free-solid-svg-icons";
    import Badge from "../badge.svelte";
    import HideOn from "../hide-on.svelte";
</script>

<nav>
    <div class="links">
        <a class="home" href="/">
            <Logo />
        </a>
        <HideOn mobile>
            <Tabs
                tabs={[
                    { name: "Emotes", pathname: "/emotes" },
                    { name: "Discover", pathname: "/discover" },
                    { name: "Store", pathname: "/store" },
                    { name: "More", pathname: "/more" },
                ]}
            />
        </HideOn>
    </div>
    <HideOn mobile>
        <SearchBar />
    </HideOn>
    <div class="user-actions">
        <button class="button center hide-on-desktop">
            <Fa icon={faSearch} fw />
        </button>
        <button class="button center">
            <Fa icon={faBell} fw />
        </button>
        <button class="button center">
            <Badge count={55}>
                <Fa icon={faMessage} fw />
            </Badge>
        </button>
        <a href="/upload" class="button center hide-on-desktop">
            <Fa icon={faPlusSquare} />
        </a>
        <a href="/upload" class="button center secondary hide-on-mobile">
            <Fa icon={faPlus} />
            Upload
        </a>
        <a href="/login" class="button primary">Login</a>
    </div> 
</nav>

<style lang="scss">
    nav {
        background-color: var(--bg-dark);
        border-bottom: 1px solid var(--border);
        padding: 0 2rem;
        height: 3.5rem;

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

    .user-actions {
        display: flex;
        gap: 0.75rem;
        align-items: center;
    }

    @media screen and (max-width: 960px) {
        nav {
            padding: 0 1rem;
            gap: 1rem;
        }

        .links {
            gap: 1rem;
        }

        .user-actions {
            gap: 0.5rem;
        }
    }
</style>
