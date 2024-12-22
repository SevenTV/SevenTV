<script lang="ts">
	import { refreshUser, user } from "$/lib/auth";
	import { Trash } from "phosphor-svelte";
	import Button from "../input/button.svelte";
	import ResponsiveImage from "../responsive-image.svelte";
	import Spinner from "../spinner.svelte";
	import { removeProfilePicture, uploadProfilePicture } from "$/lib/userMutations";
	import { t } from "svelte-i18n";
	import { untrack } from "svelte";

	let fileInput: HTMLInputElement;
	let files = $state<FileList>();

	function browse() {
		fileInput?.click();
	}

	let loading = $state(false);

	$effect(() => {
		if (files && files[0]) {
			loading = true;
			untrack(() => {
				if (!files) {
					return;
				}

				uploadProfilePicture($user!.id, files[0])
					.then((res) => {
						if (!$user || !res?.pending_profile_picture) {
							return;
						}

						$user.style.pendingProfilePictureId = res.pending_profile_picture;
					})
					.finally(() => {
						loading = false;
					});
			});
		}
	});

	let removeProfilePictureLoading = $state(false);

	async function removeUserProfilePicture() {
		if (!$user || !$user.style.activeProfilePicture) {
			return;
		}

		removeProfilePictureLoading = true;
		await removeProfilePicture($user.id);

		refreshUser();

		removeProfilePictureLoading = false;
	}
</script>

<div class="profile-picture">
	{#if $user?.style.pendingProfilePictureId}
		<div class="placeholder">
			<Spinner />
		</div>
	{:else if $user?.style.activeProfilePicture}
		<ResponsiveImage
			width={4 * 16}
			height={4 * 16}
			images={$user.style.activeProfilePicture.images}
			round
			style="grid-row: 1 / span 2;"
		/>
	{:else}
		<!-- svelte-ignore a11y_missing_attribute -->
		<img
			src={$user?.mainConnection?.platformAvatarUrl}
			width={4 * 16}
			height={4 * 16}
			style="grid-row: 1 / span 2; border-radius: 50%;"
		/>
	{/if}
	<div class="buttons">
		<input
			type="file"
			accept="image/webp, image/avif, image/avif-sequence, image/gif, image/png, image/apng, image/jls, image/jpeg, image/jxl, image/bmp, image/heic, image/heic-sequence, image/heif, image/heif-sequence, application/mp4, video/mp4, video/x-flv, video/x-matroska, video/avi, video/quicktime, video/webm, video/mp2t"
			hidden
			bind:this={fileInput}
			bind:files
		/>
		<Button
			secondary
			disabled={!!$user?.style.pendingProfilePictureId ||
				!$user?.permissions.user.useCustomProfilePicture ||
				loading}
			onclick={browse}
		>
			{$t("pages.settings.account.profile.update_profile_picture")}
		</Button>
		{#if $user?.style.activeProfilePicture}
			<Button disabled={loading || removeProfilePictureLoading} onclick={removeUserProfilePicture}>
				{#snippet icon()}
					{#if removeProfilePictureLoading}
						<Spinner />
					{:else}
						<Trash />
					{/if}
				{/snippet}
			</Button>
		{/if}
	</div>
	<span class="limits">
		{$t("file_limits.max_size", { values: { size: "7MB" } })},
		{$t("file_limits.max_resolution", { values: { width: "1000", height: "1000" } })}
	</span>
</div>

<style lang="scss">
	.profile-picture {
		display: grid;
		column-gap: 1rem;
		justify-content: start;
		grid-template-columns: repeat(2, auto);

		.placeholder {
			grid-row: 1 / span 2;

			width: 4rem;
			height: 4rem;
			background-color: var(--secondary);
			border-radius: 50%;

			display: flex;
			justify-content: center;
			align-items: center;
		}

		.buttons {
			display: flex;
			align-items: center;
			gap: 0.5rem;
		}

		.limits {
			color: var(--text-light);
			font-size: 0.75rem;
		}
	}
</style>
