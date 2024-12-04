<script lang="ts">
	import type { User } from "$/gql/graphql";
	import Button from "./input/button.svelte";
	import UserName from "./user-name.svelte";
	import UserProfilePicture from "./user-profile-picture.svelte";

	interface Props {
		user: User;
		big?: boolean;
		size?: number;
		onclick?: (e: MouseEvent) => void;
	}

	let { user, big = false, size = 2, onclick }: Props = $props();
</script>

<Button href="/users/{user.id}" {big} {onclick}>
	{#snippet icon()}
		<UserProfilePicture {user} size={size * 16} />
	{/snippet}
	<span class="user" style:color={user.highestRoleColor?.hex}>
		<UserName {user} />
	</span>
</Button>

<style lang="scss">
	.user {
		flex-grow: 1;

		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;

		font-size: 0.875rem;
		font-weight: 600;
	}
</style>
