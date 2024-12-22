<script lang="ts">
	import UserSearch from "$/components/user-search.svelte";
	import { User } from "phosphor-svelte";
</script>

<div class="layout">
	<UserSearch
		searchlimit={15}
		placeholder="Search user to inspect"
		resulthref={(user) => `/admin/users/${user.id}`}
	>
		{#snippet icon()}
			<User />
		{/snippet}
	</UserSearch>
</div>

<style lang="scss">
	.layout {
		height: 100%;
		width: 100%;
		max-width: 20rem;
		margin: 0 auto;

		display: flex;
		flex-direction: column;
		justify-content: center;
	}
</style>
