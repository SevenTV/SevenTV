<script lang="ts">
    import Logo from "$/components/icons/logo.svelte";
    import SearchBar from "$/components/nav/search-bar.svelte";
    import Tabs from "./tabs.svelte";
    import Fa from "svelte-fa";
    import { faBell, faMessage, faPlusSquare } from "@fortawesome/free-regular-svg-icons";
    import { faEllipsisV, faPlus, faSearch } from "@fortawesome/free-solid-svg-icons";
    import Badge from "../badge.svelte";
    import HideOn from "../hide-on.svelte";
    import { user, showMobileMenu } from "$/lib/stores";
    import DropDown from "../drop-down.svelte";
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
        <button class="button hide-on-desktop">
            <Fa icon={faSearch} size="1.2x" fw />
        </button>
        {#if $user}
            <button class="button">
                <Fa icon={faBell} size="1.2x" fw />
            </button>
            <button class="button">
                <Badge count={55}>
                    <Fa icon={faMessage} size="1.2x" fw />
                </Badge>
            </button>
            <a href="/upload" class="button hide-on-desktop">
                <Fa icon={faPlusSquare} size="1.2x" fw />
            </a>
            <a href="/upload" class="button secondary hide-on-mobile">
                <Fa icon={faPlus} size="1.2x" />
                Upload
            </a>
            <a href="/user/ayyybubu" class="profile hide-on-mobile">
                <img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
                <span>ayyybubu</span>
            </a>
            <button class="profile hide-on-desktop" on:click={() => ($showMobileMenu = !$showMobileMenu)}>
                <img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" />
            </button>
        {:else}
            <DropDown>
                <button class="button">
                    <Fa icon={faEllipsisV} size="1.2x" fw />
                </button>
                <svelte:fragment slot="dropdown">
                    <li>
                        <a href="/developer">Developer Portal</a>
                    </li>
                    <li>
                        <a href="/contact">Contact</a>
                    </li>
                    <li>
                        <a href="/faq">FAQ</a>
                    </li>
                    <hr />
                    <li>
                        <a href="/privacy">Privacy Policy</a>
                    </li>
                    <li>
                        <a href="/tos">Terms of Service</a>
                    </li>
                </svelte:fragment>
            </DropDown>
            <a class="button primary" href="/sign-in">Sign In</a>
        {/if}
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
        gap: 1rem;
    }

    .links {
        /* Take all available space but shrink by a very high factor */
        flex: 1 9999;

        display: flex;
        gap: 2rem;

        .home {
            display: flex;
            align-items: center;
        }
    }

    .user-actions {
        /* Take all available space but shrink by a very high factor */
        flex: 1 9999;

        display: flex;
        gap: 0.75rem;
        align-items: center;
        justify-content: flex-end;

        .profile {
            color: var(--text);

            display: flex;
            align-items: center;
            gap: 0.5rem;
            text-decoration: none;

            .profile-picture {
                width: 2rem;
                height: 2rem;

                border-radius: 50%;
                border: 2px solid var(--staff);
            }
        }
    }

    @media screen and (max-width: 960px) {
        nav {
            padding: 0 1rem;
            gap: 1rem;
        }

        .links {
            gap: 1rem;
        }
    }
</style>
