<script lang="ts">
	import type { User } from "$/gql/graphql";
	import Button from "./input/button.svelte";
	import UserProfilePicture from "./user-profile-picture.svelte";

	export let user: User;
	export let big: boolean = false;
	export let size: number = 2;
</script>

<Button href="/users/{user.id}" {big}>
	<UserProfilePicture {user} size={size * 16} slot="icon" />
	<span class="user" style:color={user.highestRoleColor?.hex}
		>{user.mainConnection?.platformDisplayName}</span
	>
</Button>

<style lang="scss">
	.user {
		flex-grow: 1;
		overflow: hidden;
		text-overflow: ellipsis;

		font-size: 0.875rem;
		font-weight: 600;
	}
</style>
