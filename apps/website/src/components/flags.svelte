<script lang="ts" module>
	// TODO: Remove `any` as soon as phosphor-svelte fully supports Svelte 5
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export const icons: { [key: string]: any } = {
		// Emote flags
		active: Lightning,
		global: GlobeSimple,
		trending: Fire,
		overlaying: StackSimple,
		private: LockSimple,
		unlisted: EyeSlash,
		personal_use_denied: EyeSlash,
		deleted: Trash,
		renamed: PencilSimple,
		personal_use_pending: ClockUser,

		// Emote set flags
		default: House,
		personal: Star,
		special: Sparkle,

		// Editor Permissions
		super_admin: Wrench,
		profile: UserIcon,
		editors: PencilSimple,
		emote_sets: FolderSimple,
		emotes: Smiley,

		// Editor states
		pending: Hourglass,
		accepted: Check,
		you_accepted: Check,
		rejected: Prohibit,
		you_rejected: Prohibit,
	};

	export const colors: { [key: string]: string } = {
		personal: "#b083f0",
		special: "#fcc200",

		active: "#529bf5",
		global: "#57ab5a",
		trending: "#e0823d",
		overlaying: "#fc8dc7",
		private: "#888888",
		unlisted: "#eb3d26",
		personal_use_denied: "#eb3d26",
		deleted: "#eb3d26",
		personal_use_pending: "#888888",

		you_rejected: "#eb3d26",
		rejected: "#eb3d26",
		pending: "#529bf5",
		accepted: "#57ab5a",
		you_accepted: "#57ab5a",
	};

	export function determineHighlightColor(flags: string[], ignoredFlags: string[] = []) {
		for (const flag of Object.keys(colors)) {
			if (flags.includes(flag) && !ignoredFlags.includes(flag)) {
				return colors[flag];
			}
		}
		return null;
	}

	export function emoteSetEmoteToFlags(
		emote: EmoteSetEmote,
		defaultSet?: string,
		editableEmoteSets?: EmoteSet[],
		setKind?: EmoteSetKind,
	): string[] {
		let flags: string[] = [];

		if (emote.alias !== emote.emote?.defaultName) flags.push("renamed");

		if (emote.emote) {
			flags = flags.concat(emoteToFlags(emote.emote, defaultSet, editableEmoteSets, setKind));
		}

		return flags;
	}

	export function emoteToFlags(
		emote: Emote,
		defaultSet?: string,
		editableEmoteSets?: EmoteSet[],
		setKind?: EmoteSetKind,
	): string[] {
		const flags: string[] = [];

		if (
			defaultSet &&
			editableEmoteSets &&
			editableEmoteSets
				.find((set) => set.id === defaultSet)
				?.emotes.items.some((e) => e.id === emote.id)
		) {
			flags.push("active");
		}

		if (emote.flags.defaultZeroWidth) flags.push("overlaying");

		if (emote.flags.private) flags.push("private");

		if (!emote.flags.publicListed) flags.push("unlisted");

		if (emote.flags.deniedPersonal) flags.push("personal_use_denied");

		if (emote.ranking && emote.ranking < 200) flags.push("trending");

		if (emote.deleted) flags.push("deleted");

		if (typeof emote?.flags?.approvedPersonal != "undefined" && !emote.flags.approvedPersonal && setKind == EmoteSetKind.Personal)
			flags.push("personal_use_pending");

		return flags;
	}

	export function emoteSetToFlags(
		set: EmoteSet,
		user?: User | null,
		defaultSet?: string,
	): string[] {
		const flags = [];

		if (user?.style.activeEmoteSetId === set.id) {
			flags.push("active");
		}

		if (set.id === defaultSet) {
			flags.push("default");
		}

		switch (set.kind) {
			case EmoteSetKind.Global:
				flags.push("global");
				break;
			case EmoteSetKind.Personal:
				flags.push("personal");
				break;
			case EmoteSetKind.Special:
				flags.push("special");
				break;
		}

		return flags;
	}

	export function editorPermissionsToFlags(permissions: UserEditorPermissions) {
		const flags = [];

		if (permissions.superAdmin) flags.push("super_admin");

		if (permissions.emoteSet.manage) flags.push("emote_sets");

		if (permissions.emote.manage) flags.push("emotes");

		if (permissions.user.manageProfile) flags.push("profile");

		if (permissions.user.manageEditors) flags.push("editors");

		return flags;
	}

	export function editorStateToFlags(state: UserEditorState, you: boolean) {
		switch (state) {
			case UserEditorState.Pending:
				return ["pending"];
			case UserEditorState.Accepted:
				return [you ? "you_accepted" : "accepted"];
			case UserEditorState.Rejected:
				return [you ? "you_rejected" : "rejected"];
			default:
				return [];
		}
	}
</script>

<script lang="ts">
	import {
		Check,
		ClockUser,
		EyeSlash,
		Fire,
		FolderSimple,
		GlobeSimple,
		Hourglass,
		House,
		Lightning,
		LockSimple,
		PencilSimple,
		Prohibit,
		Smiley,
		Sparkle,
		StackSimple,
		Star,
		Trash,
		User as UserIcon,
		Wrench,
	} from "phosphor-svelte";
	import Button from "./input/button.svelte";
	import { t } from "svelte-i18n";
	import {
		EmoteSetKind,
		UserEditorState,
		type Emote,
		type EmoteSet,
		type EmoteSetEmote,
		type User,
		type UserEditorPermissions,
	} from "$/gql/graphql";
	import type { HTMLAttributes } from "svelte/elements";

	const names: { [key: string]: string } = {
		// Emote flags
		active: $t("common.active"),
		global: $t("flags.global"),
		trending: $t("flags.trending"),
		overlaying: $t("flags.overlaying"),
		private: "Private",
		unlisted: $t("flags.unlisted"),
		personal_use_denied: $t("flags.personal_use_denied"),
		renamed: "Renamed",
		personal_use_pending: "Pending Personal Use Review",

		// Emote set flags
		default: $t("flags.default"),
		personal: $t("flags.personal"),
		special: "Special",

		// Editor Permissions
		super_admin: "Super Admin",
		profile: $t("common.profile"),
		editors: $t("common.editors"),
		emote_sets: $t("common.emote_sets", { values: { count: 2 } }),
		emotes: $t("common.emotes", { values: { count: 2 } }),
	};

	// Used for emote flags, emote set flags and permissions

	type Props = {
		iconOnly?: boolean;
		flags: string[];
		edit?: (e: MouseEvent) => void;
	} & HTMLAttributes<HTMLDivElement>;

	let { iconOnly = false, flags = [], edit, ...restProps }: Props = $props();

	let sortedFlags = $derived(
		flags.toSorted((a, b) => {
			const keys = Object.keys(icons);
			const aIndex = keys.indexOf(a);
			const bIndex = keys.indexOf(b);
			if (aIndex === -1 && bIndex === -1) {
				return 0;
			}
			if (aIndex === -1) {
				return 1;
			}
			if (bIndex === -1) {
				return -1;
			}
			return aIndex - bIndex;
		}),
	);
</script>

<div class="flags" {...restProps}>
	{#each sortedFlags as flag}
		{#if iconOnly && icons[flag]}
			<span class="flag icon-only" style="color: {colors[flag]}" title={names[flag]}>
				<!-- svelte-ignore svelte_component_deprecated -->
				<!-- Disable warning until phosphor-svelte has full Svelte 5 support -->
				<svelte:component this={icons[flag]} size={1 * 16} />
			</span>
		{:else}
			<span
				class="flag"
				class:has-icon={icons[flag]}
				style="color: {colors[flag]}; background-color: {colors[flag]}1a"
				title={names[flag]}
			>
				<!-- svelte-ignore svelte_component_deprecated -->
				<!-- Disable warning until phosphor-svelte has full Svelte 5 support -->
				<svelte:component this={icons[flag]} size={1 * 16} />
				<span class:hide-on-mobile={icons[flag]}>{names[flag] || flag.replace(/_/g, " ")}</span>
			</span>
		{/if}
	{/each}
	{#if edit}
		<Button secondary onclick={edit} title="Edit" style="padding: 0.3rem 0.5rem; border: none;">
			{#snippet icon()}
				<PencilSimple size={1 * 16} />
			{/snippet}
		</Button>
	{/if}
</div>

<style lang="scss">
	.flags {
		display: flex;
		align-items: center;
		column-gap: 0.5rem;
		row-gap: 0.3rem;
		flex-wrap: wrap;
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
			border-radius: 0;
		}
	}

	@media screen and (max-width: 960px) {
		.flag.has-icon {
			padding: 0.3rem 0.5rem;
		}
	}
</style>
