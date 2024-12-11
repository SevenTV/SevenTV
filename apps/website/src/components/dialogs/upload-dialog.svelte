<script lang="ts">
	import Dialog, { type DialogMode } from "./dialog.svelte";
	import Checkbox from "$/components/input/checkbox.svelte";
	import { Moon, Sun, Trash, UploadSimple } from "phosphor-svelte";
	import { theme, type Theme } from "$/lib/layout";
	import TagsInput from "../input/tags-input.svelte";
	import Button from "../input/button.svelte";
	import { browser } from "$app/environment";
	import Tags from "../emotes/tags.svelte";
	import TextInput from "$/components/input/text-input.svelte";
	import { t } from "svelte-i18n";
	import { upload } from "$/lib/emoteMutations";
	import { goto } from "$app/navigation";
	import Spinner from "../spinner.svelte";

	// svelte-ignore non_reactive_update
	let fileInput: HTMLInputElement;
	let dragOver = $state(false);

	let name = $state("");
	let tags: string[] = $state([]);
	let files = $state<FileList>();
	let imageSrc = $state<string>();
	let zeroWidth = $state(false);
	let privateFlag = $state(false);

	let loading = $state(false);

	let { mode = $bindable("hidden") }: { mode: DialogMode } = $props();

	function initialTheme(theme: Theme | null) {
		if (theme === "system-theme" && browser) {
			return window.matchMedia("prefers-color-scheme: dark").matches ? "dark-theme" : "light-theme";
		}

		return theme;
	}

	let previewTheme = $state(initialTheme($theme));

	$effect(() => {
		previewTheme = initialTheme($theme);
	});

	function toggleTheme() {
		previewTheme = previewTheme === "dark-theme" ? "light-theme" : "dark-theme";
	}

	$effect(() => {
		if (files && files[0]) {
			const reader = new FileReader();
			reader.onload = () => (imageSrc = reader.result as string);
			reader.readAsDataURL(files[0]);
		}
	});

	function browse() {
		fileInput?.click();
	}

	function onDrop(e: DragEvent) {
		dragOver = false;
		if (e.dataTransfer) {
			files = e.dataTransfer.files;
		}
		e.preventDefault();
	}

	function onDragOver(e: DragEvent) {
		dragOver = true;
		e.preventDefault();
	}

	function onDragLeave() {
		dragOver = false;
	}

	// let messages: string[] = $state([]);

	// let messageInput = $state("");

	// function sendMessage(e: KeyboardEvent) {
	// 	e.stopPropagation();

	// 	if (e.key === "Enter" && !e.shiftKey && messageInput.trim().length > 0) {
	// 		e.preventDefault();
	// 		messages = [...messages, messageInput];
	// 		messageInput = "";
	// 	}
	// }

	let acceptTerms = $state(false);

	async function submit() {
		if (!files || !files[0]) return;

		loading = true;

		const res = await upload(files[0], name, tags, zeroWidth, privateFlag);

		loading = false;
		mode = "hidden";

		if (res && res.emote_id) {
			goto(`/emotes/${res.emote_id}`);
		}
	}

	function reset() {
		files = undefined;
		loading = false;
	}
</script>

<Dialog width={60} bind:mode>
	<form class="grid">
		<h1 class="heading">{$t("dialogs.upload.title")}</h1>
		<section class="upload {previewTheme}" class:preview={files && files[0]}>
			{#if files && files[0] && imageSrc}
				<span class="name">{name}</span>
				<Tags {tags} />
				<div class="previews">
					{#each [32, 64, 96, 128] as resolution}
						<div class="preview">
							<img src={imageSrc} width={resolution} alt="Upload Preview" />
							<span class="size-text">{resolution}x{resolution}</span>
						</div>
					{/each}
				</div>
				<div class="buttons">
					<Button secondary onclick={toggleTheme}>
						{#snippet icon()}
							{#if previewTheme === "dark-theme"}
								<Moon />
							{:else}
								<Sun />
							{/if}
						{/snippet}
					</Button>
					<Button secondary onclick={reset}>
						{#snippet icon()}
							<Trash />
						{/snippet}
					</Button>
				</div>
			{:else}
				<div
					class="file"
					role="button"
					tabindex="-1"
					class:drag-over={dragOver}
					ondrop={onDrop}
					ondragover={onDragOver}
					ondragleave={onDragLeave}
				>
					<input
						type="file"
						accept="image/webp, image/avif, image/avif-sequence, image/gif, image/png, image/apng, image/jls, image/jpeg, image/jxl, image/bmp, image/heic, image/heic-sequence, image/heif, image/heif-sequence, application/mp4, video/mp4, video/x-flv, video/x-matroska, video/avi, video/quicktime, video/webm, video/mp2t"
						hidden
						bind:this={fileInput}
						bind:files
					/>
					<UploadSimple size={1.5 * 16} color="var(--text-light)" />
					<h2>{$t("dialogs.upload.drag_drop")}</h2>
					<Button secondary onclick={browse}>{$t("dialogs.upload.browse_files")}</Button>
					<span class="details">
						{$t("file_limits.max_size", { values: { size: "7MB" } })}
						<br />
						{$t("file_limits.max_resolution", { values: { width: "1000", height: "1000" } })}
						<br />
						{$t("file_limits.max_frames", { values: { count: "1000" } })}
					</span>
				</div>
			{/if}
		</section>
		<section class="left">
			<div class="inputs">
				<TextInput placeholder={$t("labels.emote_name")} bind:value={name}>
					<span class="label">{$t("labels.emote_name")}</span>
				</TextInput>
				<TagsInput bind:tags>
					<span class="label">{$t("labels.tags")}</span>
				</TagsInput>
				<!-- <TextInput placeholder={$t("labels.search_users", { values: { count: 2 } })}>
					<span class="label">{$t("labels.emote_attribution")}</span>
					{#snippet icon()}
						<User />
					{/snippet}
				</TextInput> -->
				<Checkbox bind:value={zeroWidth}>{$t("flags.overlaying")}</Checkbox>
				<Checkbox bind:value={privateFlag}>Private</Checkbox>
			</div>
			<div class="footer">
				<Checkbox bind:value={acceptTerms}>{$t("dialogs.upload.accept_rules")}</Checkbox>
				<div class="buttons">
					<Button secondary onclick={() => (mode = "hidden")}>
						{$t("dialogs.upload.discard")}
					</Button>
					{#snippet loadingSpinner()}
						<Spinner />
					{/snippet}
					<Button
						primary
						submit
						disabled={!acceptTerms || !name}
						onclick={submit}
						icon={loading ? loadingSpinner : undefined}
					>
						{$t("dialogs.upload.upload")}
					</Button>
				</div>
			</div>
		</section>
		<!-- <section class="chat">
			<div class="messages">
				{#each messages as message}
					<span class="message">
						<span class="username">ayyybubu</span>: {message}
					</span>
				{/each}
			</div>
			<input
				type="text"
				placeholder={$t("dialogs.upload.chat")}
				bind:value={messageInput}
				onkeydown={sendMessage}
			/>
		</section> -->
	</form>
</Dialog>

<style lang="scss">
	.grid {
		padding: 1rem;

		display: grid;
		grid-template-areas: "heading heading" "left upload" "left upload";
		grid-template-columns: 18.5rem 1fr;
		grid-template-rows: auto auto 1fr;
		gap: 1rem;
	}

	.heading {
		grid-area: heading;

		font-size: 1.25rem;
		font-weight: 600;
	}

	section {
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}

	.label {
		font-size: 0.75rem;
		color: var(--text-light);
	}

	.upload {
		grid-area: upload;
		padding: 1.25rem;
		min-height: 17.5rem;
		color: var(--text);

		.file {
			width: 100%;
			height: 100%;
			background-image: url("data:image/svg+xml,%3csvg width='100%25' height='100%25' xmlns='http://www.w3.org/2000/svg'%3e%3crect width='100%25' height='100%25' fill='none' rx='8' ry='8' stroke='%23333333' stroke-width='3' stroke-dasharray='8%2c 10' stroke-dashoffset='13' stroke-linecap='butt'/%3e%3c/svg%3e");
			border-radius: 0.5rem;
			padding: 1rem 1.25rem;

			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;
			gap: 0.75rem;

			text-align: center;

			h2 {
				color: var(--text);
				font-size: 1rem;
				font-weight: 500;
			}

			.details {
				color: var(--text-light);
				font-size: 0.6875rem;
			}

			&.drag-over {
				border-style: solid;
				border-color: var(--primary);

				background-color: var(--bg-light);
			}
		}

		.buttons {
			position: absolute;
			top: 1rem;
			right: 1rem;

			display: flex;
			gap: 0.5rem;
			align-items: center;
		}

		&.preview {
			display: flex;
			flex-direction: column;
			justify-content: space-between;
			align-items: center;
			gap: 0.5rem;

			position: relative;

			.name {
				font-size: 1.25rem;
				font-weight: 600;
			}

			.previews {
				display: flex;
				justify-content: center;
				align-items: flex-end;
				gap: 1rem;

				.preview {
					display: flex;
					flex-direction: column;
					gap: 1rem;
					align-items: center;
				}

				.size-text {
					color: var(--text-light);
					font-size: 0.75rem;
				}
			}
		}
	}

	.left {
		grid-area: left;
		padding: 1rem 1.25rem;

		display: flex;
		flex-direction: column;
		justify-content: space-between;
		gap: 1rem;

		.inputs {
			display: flex;
			flex-direction: column;
			gap: 1rem;
		}

		.footer {
			display: flex;
			flex-direction: column;
			gap: 1rem;
		}

		.buttons {
			display: flex;
			gap: 0.5rem;
			justify-content: flex-end;
			flex-wrap: wrap;
		}
	}

	// .chat {
	// 	grid-area: chat;
	// 	padding: 1rem;
	// 	min-height: 15rem;

	// 	display: flex;
	// 	flex-direction: column;
	// 	gap: 0.7rem;

	// 	input {
	// 		font-size: 0.8125rem;
	// 		font-weight: 400;

	// 		border-color: var(--border-active);
	// 		padding-block: 0.6rem;
	// 		background-color: var(--bg-medium);

	// 		&::placeholder {
	// 			opacity: 1;
	// 			font-weight: 400;
	// 		}
	// 	}
	// }

	// .messages {
	// 	flex-grow: 1;
	// 	flex-basis: 0;
	// 	overflow: hidden;

	// 	padding-left: 0.6rem;

	// 	display: flex;
	// 	flex-direction: column;
	// 	justify-content: flex-end;
	// 	gap: 0.6rem;
	// }

	// .message {
	// 	font-size: 0.8125rem;

	// 	.username {
	// 		color: var(--primary);
	// 		font-weight: 700;
	// 	}
	// }

	@media screen and (max-width: 960px) {
		.grid {
			grid-template-areas: "heading" "upload" "left";
			grid-template-columns: 1fr;
		}
	}
</style>
