<script lang="ts">
	import type { User } from "$/gql/graphql";
	import ResponsiveImage from "./responsive-image.svelte";

	type Props = {
		user?: User;
		size?: number;
		style?: string;
	};

	let { user, size = 44, style }: Props = $props();
</script>

{#if !user}
	<div class="placeholder loading-animation" style:width="{size}px" style:height="{size}px" {style}></div>
{:else}
	{#if user.style.activeProfilePicture}
		<ResponsiveImage
			width={size}
			height={size}
			images={user.style.activeProfilePicture.images}
			round
			borderColor={user.highestRoleColor?.hex}
			{style}
		/>
	{:else}
		<!-- svelte-ignore a11y_missing_attribute -->
		<img
			src={user.mainConnection?.platformAvatarUrl}
			style:border-color={user.highestRoleColor?.hex ?? "transparent"}
			width={size}
			height={size}
			class="profile-picture"
			{style}
		/>
	{/if}
{/if}

<style lang="scss">
	.profile-picture {
		border-radius: 50%;
		border: 2px solid;
	}

	.placeholder {
		border-radius: 50%;
		background-color: var(--preview);
	}
</style>
