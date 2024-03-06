<svelte:head>
	<title>Account Settings - 7TV</title>
</svelte:head>

<section>
	<div class="heading">
		<h2>Profile</h2>
		<span class="details">Customize preferences and manage details to suit your individual needs</span>
	</div>
	<div class="content">
	</div>
</section>

<section>
	<div class="heading">
		<h2>Connections</h2>
		<span class="details">Manage and link external accounts to streamline access and enhance interoperability within the platform</span>
	</div>
	<div class="content"></div>
</section>

<section>
	<div class="heading">
		<h2>Security</h2>
		<span class="details">Manage and customize your account security with options like password updates and two-factor authentication</span>
	</div>
	<div class="content"></div>
</section>

<style lang="scss">
	section {
		margin-bottom: 1rem;

		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.heading {
		h2 {
			font-size: 1.125rem;
			font-weight: 600;
		}

		.details {
			color: var(--text-light);
			font-size: 0.875rem;
		}
	}

	.content {
		padding: 1rem;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;
	}
</style>
