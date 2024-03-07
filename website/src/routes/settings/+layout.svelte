<script lang="ts">
	import { DialogMode } from "$/components/dialogs/dialog.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import TabLink from "$/components/tab-link.svelte";
	import { signInDialogMode, user } from "$/lib/stores";
    import { Key, PencilSimple, Bell, CreditCard, Prohibit, MagnifyingGlass } from "phosphor-svelte";

    $: if (!$user && !$signInDialogMode) {
        $signInDialogMode = DialogMode.ShownWithoutClose;
    }
</script>

<svelte:head>
    <title>Settings - 7TV</title>
</svelte:head>

{#if $user}
    <div class="side-bar-layout">
        <aside class="side-bar">
            <h1>Settings</h1>
            <TextInput placeholder="Search">
                <MagnifyingGlass slot="icon" />
            </TextInput>
            <nav class="link-list">
                <TabLink title="Account" href="/settings" big>
                    <Key />
                    <Key weight="fill" slot="active" />
                </TabLink>
                <TabLink title="Editors" href="/settings/editors" big>
                    <PencilSimple />
                    <PencilSimple weight="fill" slot="active" />
                </TabLink>
                <TabLink title="Notifications" href="/settings/notifications" big>
                    <Bell />
                    <Bell weight="fill" slot="active" />
                </TabLink>
                <TabLink title="Blocking" href="/settings/blocking" big>
                    <Prohibit />
                    <Prohibit weight="fill" slot="active" />
                </TabLink>
                <TabLink title="Billing" href="/settings/billing" big>
                    <CreditCard />
                    <CreditCard weight="fill" slot="active" />
                </TabLink>
            </nav>
            <div class="account hide-on-mobile">
                <img class="profile-picture" src="/test-profile-pic.jpeg" alt="profile" width="{2.5 * 16}" height="{2.5 * 16}" />
                <span class="name">ayyybubu</span>
            </div>
        </aside>
        <div class="content">
            <div class="width-wrapper">
                <slot />
            </div>
        </div>
    </div>
{/if}

<style lang="scss">
    .account {
        margin-top: auto;

        display: flex;
        align-items: center;
        gap: 0.5rem;

        .profile-picture {
            border-radius: 50%;
            border: 2px solid var(--staff);
        }

        .name {
            color: var(--staff);
            font-weight: 600;
        }
    }

    // Only desktop
	@media screen and (min-width: 961px) {
		.content {
			overflow: auto;
			overflow: overlay;
			scrollbar-gutter: stable;
        }
    }

    .width-wrapper {
        margin-inline: auto;
        max-width: 80rem;

        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
</style>
