<script lang="ts">
	import Dialog, { DialogMode } from "./dialog.svelte";
	import Checkbox from "$/components/checkbox.svelte";
	import { Moon, Sun, Trash, UploadSimple, User } from "phosphor-svelte";
	import { Theme, theme } from "$/lib/stores";
	import TagsInput from "../input/tags-input.svelte";
	import Button from "../button.svelte";
	import ImagePreview from "../image-preview.svelte";
	import { browser } from "$app/environment";
	import Tags from "../emotes/tags.svelte";
	import TextInput from "$/components/input/text-input.svelte";

	let fileInput: HTMLInputElement;
	let dragOver = false;

	let name: string;
	let tags: string[] = [];
	let files: FileList | null = null;
	let imageSrc: string;

	export let mode: DialogMode = DialogMode.Hidden;

	function initialTheme(theme: Theme | null) {
		if (theme === Theme.System && browser) {
			return window.matchMedia("prefers-color-scheme: dark") ? Theme.Dark : Theme.Light;
		}
		return theme;
	}

	$: previewTheme = initialTheme($theme);

	function toggleTheme() {
		previewTheme = previewTheme === Theme.Dark ? Theme.Light : Theme.Dark;
	}

	$: if (files && files[0]) {
		const reader = new FileReader();
		reader.onload = () => (imageSrc = reader.result as string);
		reader.readAsDataURL(files[0]);
	}

	function browse() {
		fileInput.click();
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

	let messages: string[] = [];

	let messageInput = "";

	function sendMessage(e: KeyboardEvent) {
		if (e.key === "Enter" && !e.shiftKey && messageInput.trim().length > 0) {
			e.preventDefault();
			messages = [...messages, messageInput];
			messageInput = "";
		}
	}
</script>

<Dialog width={60} bind:mode>
	<div class="grid">
		<h1 class="heading">Upload emote</h1>
		<section class="upload {previewTheme}" class:preview={files && files[0]}>
			{#if files && files[0] && imageSrc}
				<span class="name">{name || "Untitled"}</span>
				<Tags {tags} />
				<div class="previews">
					<ImagePreview size={32} src={imageSrc} />
					<ImagePreview size={64} src={imageSrc} />
					<ImagePreview size={96} src={imageSrc} />
					<ImagePreview size={128} src={imageSrc} />
				</div>
				<div class="buttons">
					<Button secondary on:click={toggleTheme}>
						<svelte:fragment slot="icon">
							{#if previewTheme === Theme.Dark}
								<Moon />
							{:else}
								<Sun />
							{/if}
						</svelte:fragment>
					</Button>
					<Button secondary on:click={() => (files = null)}>
						<Trash slot="icon" />
					</Button>
				</div>
			{:else}
				<div
					class="file"
					role="button"
					tabindex="-1"
					class:drag-over={dragOver}
					on:drop={onDrop}
					on:dragover={onDragOver}
					on:dragleave={onDragLeave}
				>
					<input
						type="file"
						accept="image/webp, image/avif, image/avif-sequence, image/gif, image/png, image/apng, image/jls, image/jpeg, image/jxl, image/bmp, image/heic, image/heic-sequence, image/heif, image/heif-sequence, application/mp4, video/mp4, video/x-flv, video/x-matroska, video/avi, video/quicktime, video/webm, video/mp2t"
						hidden
						bind:this={fileInput}
						bind:files
					/>
					<UploadSimple size="1.5rem" color="var(--text-light)" />
					<h2>Drag & drop to upload, or</h2>
					<Button secondary on:click={browse}>Browse Files</Button>
					<span class="details">
						7MB max file size
						<br />
						1000 x 1000px max resolution
						<br />
						1000 frames max
					</span>
				</div>
			{/if}
		</section>
		<section class="left">
			<div class="inputs">
				<TextInput placeholder="Enter text" bind:value={name}>
					<span class="label">Emote Name</span>
				</TextInput>
				<TagsInput bind:tags>
					<span class="label">Tags</span>
				</TagsInput>
				<TextInput placeholder="Search users">
					<span class="label">Emote Attribution</span>
					<User slot="icon" />
				</TextInput>
				<Checkbox>Zero-Width</Checkbox>
				<Checkbox>Private</Checkbox>
			</div>
			<div class="buttons">
				<Button secondary on:click={() => (mode = DialogMode.Hidden)}>Discard</Button>
				<Button primary>Upload</Button>
			</div>
		</section>
		<section class="chat">
			<div class="messages">
				{#each messages as message}
					<span class="message">
						<span class="username">ayyybubu</span>: {message}
					</span>
				{/each}
			</div>
			<input
				type="text"
				placeholder="Send a message"
				bind:value={messageInput}
				on:keydown|stopPropagation={sendMessage}
			/>
		</section>
	</div>
</Dialog>

<style lang="scss">
	.grid {
		padding: 1rem;

		display: grid;
		grid-template-areas: "heading heading" "left upload" "left chat";
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

				span {
					color: var(--primary);
				}
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

		.buttons {
			display: flex;
			gap: 0.5rem;
			justify-content: flex-end;
			flex-wrap: wrap;
		}
	}

	.chat {
		grid-area: chat;
		padding: 1rem;
		min-height: 15rem;

		display: flex;
		flex-direction: column;
		gap: 0.7rem;

		input {
			font-size: 0.8125rem;
			font-weight: 400;

			border-color: var(--border-active);
			padding-block: 0.6rem;
			background-color: var(--bg-medium);

			&::placeholder {
				opacity: 1;
				font-weight: 400;
			}
		}
	}

	.messages {
		flex-grow: 1;
		flex-basis: 0;
		overflow: hidden;

		padding-left: 0.6rem;

		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 0.6rem;
	}

	.message {
		font-size: 0.8125rem;

		.username {
			color: var(--primary);
			font-weight: 700;
		}
	}

	@media screen and (max-width: 960px) {
		.grid {
			grid-template-areas: "heading" "upload" "left" "chat";
			grid-template-columns: 1fr;
		}
	}
</style>
