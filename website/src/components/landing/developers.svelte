<script lang="ts">
	import { Code } from "phosphor-svelte";
    import GithubLogo from "../icons/github-logo.svelte";
	import Button from "../input/button.svelte";
</script>

<section class="developers">
    <div class="column">
        <GithubLogo size="2.5rem" />
        <h2>Source-Available</h2>
        <p>Our entire codebase is available on GitHub with a source-available license. Anyone can view and contribute.</p>
        <Button primary big href={import.meta.env.VITE_GITHUB_LINK}>Contribute</Button>
    </div>
    <div class="column">
        <Code size="2.5rem" />
        <h2>Developer Portal</h2>
        <p>This area exists to provide documentation for developers to integrate with our API. </p>
        <Button primary big href={import.meta.env.VITE_DEVELOPER_PORTAL}>Get Started</Button>
    </div>
</section>

<style lang="scss">
    .developers {
        display: flex;
        gap: 4rem 2rem;
        flex-wrap: wrap;
        justify-content: space-evenly;
    }

    .column {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;

        max-width: 32rem;
        text-align: center;

        h2 {
            font-size: 3rem;
            font-weight: 700;
        }

        p {
            flex-grow: 1;

            font-size: 1.125rem;
            font-weight: 400;
            line-height: 1.8rem;
        }
    }
</style>
