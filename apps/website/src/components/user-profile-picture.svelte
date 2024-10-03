<script lang="ts">
	import type { User } from "$/gql/graphql";
	import ResponsiveImage from "./responsive-image.svelte";

	export let user: User;
	export let size: number = 44;
</script>

{#if user.style.activeProfilePicture}
	<ResponsiveImage
		width={size}
		height={size}
		style={"border-radius: 50%; border: 2px solid; border-color: " + (user.highestRoleColor?.hex ?? "transparent")}
		images={user.style.activeProfilePicture.images}
		alt={user.mainConnection?.platformDisplayName ?? "profile"}
		{...$$restProps}
	/>
{:else}
	<img
		src={user.mainConnection?.platformAvatarUrl}
		style:border-color={user.highestRoleColor?.hex ?? "transparent"}
		width={size}
		height={size}
		alt={user.mainConnection?.platformDisplayName ?? "profile"}
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
