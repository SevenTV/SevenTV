<script lang="ts">
	import type { User } from "$/gql/graphql";
	import ResponsiveImage from "./responsive-image.svelte";
	import { type HTMLAttributes } from "svelte/elements";

	type Props = {
		user: User;
		size?: number;
	} & HTMLAttributes<HTMLImageElement> &
		HTMLAttributes<HTMLPictureElement>;

	let { user, size = 44, ...restProps }: Props = $props();
</script>

{#if user.style.activeProfilePicture}
	<ResponsiveImage
		width={size}
		height={size}
		images={user.style.activeProfilePicture.images}
		round
		borderColor={user.highestRoleColor?.hex}
		{...restProps}
	/>
{:else}
	<img
		src={user.mainConnection?.platformAvatarUrl}
		style:border-color={user.highestRoleColor?.hex ?? "transparent"}
		width={size}
		height={size}
		class="profile-picture"
		{...restProps}
	/>
{/if}

<style lang="scss">
	.profile-picture {
		border-radius: 50%;
		border: 2px solid;
	}
</style>
