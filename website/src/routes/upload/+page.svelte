<script lang="ts">
	import Checkbox from "$/components/checkbox.svelte";
	import SearchBar from "$/components/search-bar.svelte";
	import { faArrowUpFromBracket } from "@fortawesome/pro-solid-svg-icons";
	import Fa from "svelte-fa";

	let fileInput: HTMLInputElement;
	let dragOver = false;

	function browse() {
		fileInput.click();
	}

	function onDrop(e: DragEvent) {
		dragOver = false;
		e.preventDefault();
	}

	function onDragOver(e: DragEvent) {
		e.preventDefault();
	}

	function onFormatsClick(e: MouseEvent) {
		alert("Supported formats: PNG");
		e.stopPropagation();
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

<svelte:head>
	<title>Emote Upload - 7TV</title>
</svelte:head>

<div class="grid">
	<section class="upload">
		<button class="file" class:drag-over={dragOver} on:click={browse} on:drop={onDrop} on:dragover={onDragOver} on:dragenter={() => (dragOver = true)} on:dragleave={() => (dragOver = false)}>
			<input type="file" hidden bind:this={fileInput} />
			<Fa icon={faArrowUpFromBracket} size="1.2x" />
			<h2>Drop your emote here, or <button>Browse</button></h2>
			<span class="details">
				Maximum of 7MB
				<br />
				Resolution below 1000x1000 px
				<br />
				Frames below 1000
			</span>
			<button on:click={onFormatsClick}>Supported formats</button>
		</button>
	</section>
	<section class="inputs">
		<label>
			Emote Name
			<input type="text" placeholder="Enter text" />
		</label>
		<label>
			Tags
			<input type="text" placeholder="Enter text" />
		</label>
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			Emote Attribution
			<SearchBar grow />
		</label>
		<Checkbox label="Zero-Width" />
		<Checkbox label="Private" />
	</section>
	<section class="chat">
		<div class="messages">
			{#each messages as message}
				<span class="message">
					<span class="username">ayyybubu</span>: {message}
				</span>
			{/each}
		</div>
		<input type="text" placeholder="Send a message" bind:value={messageInput} on:keydown|stopPropagation={sendMessage} />
	</section>
</div>

<style lang="scss">
	.grid {
		display: grid;
		grid-template-areas: "inputs upload" "inputs chat";
		grid-template-columns: 22.5rem 1fr;
		grid-template-rows: 1fr 1fr;
		gap: 1rem;

		height: 100%;
		padding: 1rem 2rem;
	}

	section {
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}

	label {
		color: var(--text-lighter);
		font-size: 0.75rem;

		input, :global(search) {
			margin-top: 0.25rem;
		}
	}

	.upload {
		grid-area: upload;
		padding: 1.5rem;

		.file {
			width: 100%;
			height: 100%;
			border: 1px dashed var(--text-lighter);
			border-radius: 0.5rem;

			display: flex;
			flex-direction: column;
			align-items: center;
			justify-content: center;
			gap: 1rem;

			color: var(--text-lighter);
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
				font-size: 0.875rem;
			}

			button {
				color: var(--primary);
				font-weight: 500;
				padding: 0.5rem;
				margin: -0.5rem;
				
				&:hover, &:focus-visible {
					text-decoration: underline;
				}
			}

			&.drag-over {
				border-style: solid;
				border-color: var(--primary);

				background-color: var(--bg-light);
			}
		}
	}

	.inputs {
		grid-area: inputs;

		padding: 2rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.chat {
		grid-area: chat;
		padding: 0.6rem;

		display: flex;
		flex-direction: column;
		gap: 0.7rem;

        input {
            font-size: 0.8125rem;
            font-weight: 400;

            border-radius: 0.25rem;
            border-color: var(--border);
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
			grid-template-areas: "upload" "inputs" "chat";
			grid-template-columns: 1fr;
			grid-template-rows: repeat(3, auto);
		}
	}
</style>
