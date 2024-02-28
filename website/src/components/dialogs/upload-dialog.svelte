<script lang="ts">
	import Dialog from "./dialog.svelte";
	import Checkbox from "$/components/checkbox.svelte";
	import SearchBar from "$/components/search-bar.svelte";
	import { Moon, Sun, UploadSimple } from "phosphor-svelte";
	import { showUploadDialog } from "$/lib/stores";
	import TagsInput from "../tags-input.svelte";
	import Button from "../button.svelte";
	import ImagePreview from "../image-preview.svelte";
	import Tag from "../emotes/tag.svelte";

	let fileInput: HTMLInputElement;
	let dragOver = false;

	let name: string;
	let tags: string[] = [];
	let files: FileList;
	let imageSrc: string;
	let bgLight: boolean = false;

	$: if (files && files[0]) {
		const reader = new FileReader();
		reader.onload = () => (imageSrc = reader.result as string);
		reader.readAsDataURL(files[0]);
	}

	function close() {
		$showUploadDialog = false;
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

<Dialog width={60} on:close={close}>
	<div class="grid">
		<h1 class="heading">Upload emote</h1>
		<section class="upload" class:preview={files && files[0]} class:bg-light={bgLight}>
			{#if files && files[0] && imageSrc}
				<span class="name">{name || "Untitled"}</span>
				<div class="tags">
					{#each tags as tag}
						<Tag>{tag}</Tag>
					{/each}
				</div>
				<div class="previews">
					<ImagePreview size={32} src={imageSrc} />
					<ImagePreview size={64} src={imageSrc} />
					<ImagePreview size={96} src={imageSrc} />
					<ImagePreview size={128} src={imageSrc} />
				</div>
				<Button
					primary
					on:click={() => (bgLight = !bgLight)}
					style="position: absolute; top: 1rem; right: 1rem;"
				>
					<svelte:fragment slot="icon">
						{#if bgLight}
							<Sun />
						{:else}
							<Moon />
						{/if}
					</svelte:fragment>
				</Button>
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
					<h2>Drop & drop to upload, or</h2>
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
				<div class="field">
					<span class="label">Emote Name</span>
					<input type="text" placeholder="Enter text" bind:value={name} />
				</div>
				<div class="field">
					<span class="label">Tags</span>
					<TagsInput bind:tags />
				</div>
				<div class="field">
					<span class="label">Emote Attribution</span>
					<SearchBar grow />
				</div>
				<Checkbox label="Zero-Width" />
				<Checkbox label="Private" />
			</div>
			<div class="buttons">
				<Button secondary on:click={close}>Discard</Button>
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
		display: grid;
		grid-template-areas: "heading heading" "left upload" "left chat";
		grid-template-columns: 18.5rem 1fr;
		grid-template-rows: auto auto 1fr;
		gap: 1rem;

		height: 100%;
		padding: 1rem;
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

	.field {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;

		.label {
			color: var(--text-light);
			font-size: 0.75rem;
		}
	}

	.upload {
		grid-area: upload;
		padding: 1.25rem;
		min-height: 17.5rem;

		&.bg-light {
			background-color: white;
			color: black;
		}

		.file {
			width: 100%;
			height: 100%;
			border: 1px dashed var(--text-light);
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

			.tags {
				display: flex;
				gap: 0.5rem;
				flex-wrap: wrap;
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
