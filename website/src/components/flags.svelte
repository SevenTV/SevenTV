<script lang="ts">
	import {
		Eye,
		Fire,
		FolderSimple,
		GlobeSimple,
		PencilSimple,
		Plus,
		SealCheck,
		Smiley,
		StackSimple,
		User,
	} from "phosphor-svelte";
	import Button from "./button.svelte";

	// Used for emote flags, emote set flags and permissions

	const icons: { [key: string]: typeof GlobeSimple } = {
		// Emote / emote set flags
		global: GlobeSimple,
		trending: Fire,
		overlay: StackSimple,
		verified: SealCheck,
		public: Eye,

		// Permissions
		profile: User,
		editors: PencilSimple,
		emote_sets: FolderSimple,
		emotes: Smiley,
	};

	const details: { [key: string]: string } = {
		// Emote / emote set flags
		global: "Global",
		trending: "Trending",
		overlay: "Overlaying",
		verified: "Verified",
		public: "Publicly Listed",

		// Permissions
		profile: "Profile",
		editors: "Editors",
		emote_sets: "Emote Sets",
		emotes: "Emotes",
	};

	export let flags: string[] = [];
	export let add: ((e: MouseEvent) => void) | null = null;
</script>

<div class="flags" {...$$restProps}>
	{#each flags as flag}
		<span class="flag" class:has-icon={icons[flag]} title={details[flag]}>
			<svelte:component this={icons[flag]} size="1rem" />
			<span class:hide-on-mobile={icons[flag]}>{flag.replace("_", " ")}</span>
		</span>
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
	}

	@media screen and (max-width: 960px) {
		.flag.has-icon {
			padding: 0.3rem 0.5rem;
		}
	}
</style>
