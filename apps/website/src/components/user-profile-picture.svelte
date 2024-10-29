<script lang="ts">
	import type { User } from "$/gql/graphql";
	import ResponsiveImage from "./responsive-image.svelte";

	export let user: User;
	export let size: number = 44;

	$: alt = user.mainConnection ? `${user.mainConnection?.platformDisplayName}'s Profile Picture` : "Profile Picture";
</script>

{#if user.style.activeProfilePicture}
	<ResponsiveImage
		width={size}
		height={size}
		images={user.style.activeProfilePicture.images}
		{alt}
		round
		borderColor={user.highestRoleColor?.hex}
		{...$$restProps}
	/>
{:else}
	<img
		src={user.mainConnection?.platformAvatarUrl}
		style:border-color={user.highestRoleColor?.hex ?? "transparent"}
		width={size}
		height={size}
		{alt}
		class="profile-picture"
		{...$$restProps}
	/>
{/if}

<style lang="scss">
	.profile-picture {
		border-radius: 50%;
		border: 2px solid;
	}
</style>
