<script lang="ts">
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

<div class="chat">
    <div class="messages">
        {#each messages as message}
            <span class="message">
                <span class="username">ayyybubu</span>: {message}
            </span>
        {/each}
    </div>
	<input type="text" placeholder="Send a message" bind:value={messageInput} on:keydown|stopPropagation={sendMessage} />
</div>

<style lang="scss">
	.chat {
        height: 100%;
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
        padding-left: 0.6rem;

        display: flex;
		flex-direction: column;
		gap: 0.6rem;
        justify-content: flex-end;
    }

	.message {
		font-size: 0.8125rem;

		.username {
			color: var(--primary);
			font-weight: 700;
		}
	}
</style>
