<script lang="ts" context="module">
	export const icons: { [key: string]: typeof GlobeSimple } = {
		// Emote flags
		active: Lightning,
		global: GlobeSimple,
		trending: Fire,
		overlaying: StackSimple,
		verified: SealCheck,
		public: Eye,

		// Emote set flags
		default: House,
		personal: Star,

		// Permissions
		profile: User,
		editors: PencilSimple,
		emote_sets: FolderSimple,
		emotes: Smiley,
	};

	export const colors: { [key: string]: string } = {
		personal: "#b083f0",

		active: "#529bf5",
		global: "#57ab5a",
		trending: "#e0823d",
		overlaying: "#fc8dc7",
		verified: "#fc8dc7",
		public: "#ffffff",
	};

	export function determineHighlightColor(flags: string[], ignoredFlags: string[] = []) {
		for (const flag of Object.keys(colors)) {
			if (flags.includes(flag) && !ignoredFlags.includes(flag)) {
				return colors[flag];
			}
		}
		return null;
	}
</script>

<script lang="ts">
	import {
		Eye,
		Fire,
		FolderSimple,
		GlobeSimple,
		House,
		Lightning,
		PencilSimple,
		Plus,
		SealCheck,
		Smiley,
		StackSimple,
		Star,
		User,
	} from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import { t } from "svelte-i18n";

	const names: { [key: string]: string } = {
		// Emote flags
		active: $t("common.active"),
		global: $t("flags.global"),
		trending: $t("flags.trending"),
		overlaying: $t("flags.overlaying"),
		verified: $t("flags.verified"),
		public: $t("flags.public"),

		// Emote set flags
		default: $t("flags.default"),
		personal: $t("flags.personal"),

		// Permissions
		profile: $t("common.profile"),
		editors: $t("common.editors"),
		emote_sets: $t("common.emote_sets", { values: { count: 2 } }),
		emotes: $t("common.emotes", { values: { count: 2 } }),
	};

	// Used for emote flags, emote set flags and permissions

	export let iconOnly: boolean = false;

	export let flags: string[] = [];
	export let add: ((e: MouseEvent) => void) | null = null;

	$: flags.sort((a, b) => {
		const keys = Object.keys(icons);
		return keys.indexOf(a) - keys.indexOf(b);
	});
</script>

<div class="flags" {...$$restProps}>
	{#each flags as flag}
		{#if iconOnly && icons[flag]}
			<span class="flag icon-only" style="color: {colors[flag]}">
				<svelte:component this={icons[flag]} size="1rem" />
			</span>
		{:else}
			<span
				class="flag"
				class:has-icon={icons[flag]}
				style="color: {colors[flag]}; background-color: {colors[flag]}1a"
			>
				<svelte:component this={icons[flag]} size="1rem" />
				<span class:hide-on-mobile={icons[flag]}>{names[flag] || flag.replace("_", " ")}</span>
			</span>
		{/if}
	{/each}
	{#if add}
		<Button secondary on:click={add} title="Add" style="padding: 0.3rem 0.5rem; border: none;">
			<Plus size="1rem" slot="icon" />
		</Button>
	{/if}
</div>

<style lang="scss">
	.flags {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.flag {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.3rem 0.5rem;
		background-color: var(--secondary);
		border-radius: 0.5rem;

		color: var(--text-light);
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: capitalize;

		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;

		&.has-icon {
			padding-left: 0.62rem;
		}

		&.icon-only {
			padding: 0;
			background: none;
		}
	}

	@media screen and (max-width: 960px) {
		.flag.has-icon:not(.icon-only) {
			padding: 0.3rem 0.5rem;
		}
	}
</style>
