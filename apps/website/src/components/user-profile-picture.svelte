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
		style="border-radius: 50%; border: 2px solid var(--staff);"
		images={user.style.activeProfilePicture.images}
		alt={user.mainConnection?.platformDisplayName ?? "profile"}
	/>
{:else}
	<img
		src={user.mainConnection?.platformAvatarUrl}
		width={size}
		height={size}
		alt={user.mainConnection?.platformDisplayName ?? "profile"}
		class="profile-picture"
	/>
{/if}

<style lang="scss">
	.profile-picture {
		border-radius: 50%;
		border: 2px solid var(--staff);
	}
</style>
