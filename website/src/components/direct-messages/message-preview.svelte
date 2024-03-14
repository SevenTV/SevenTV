<script lang="ts">
	import moment from 'moment/min/moment-with-locales';
</script>

<a class="preview" href="/direct-messages/ayyybubu">
	<div class="profile-picture"></div>
	<span class="name">ayyybubu</span>
	<span class="details"
		>Lorem ipsum, dolor sit amet consectetur adipisicing elit. Dolor, cumque enim qui nam dicta
		ipsum illo animi inventore aut libero sit, sed consequatur earum quidem quia rem! Pariatur, ut
		quo.</span
	>
	<span class="details from-now">{moment("1970-01-01T00:00:00").fromNow()}</span>
</a>

<style lang="scss">
	.preview {
		display: grid;
		grid-template-columns: auto 1fr auto;
		column-gap: 1rem;
		align-items: center;

		text-decoration: none;
		color: var(--text);
		padding: 0.5rem;
		border-radius: 0.5rem;

		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}
	}

	.profile-picture {
		grid-row: span 2;

		width: 3rem;
		height: 3rem;
		border-radius: 50%;
		background-color: var(--secondary);
	}

	.name {
		grid-column: 2 / -1;

		font-size: 0.875rem;
		font-weight: 500;
	}

	.details {
		color: var(--text-light);
		font-size: 0.75rem;

		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.from-now {
		justify-self: end;
	}
</style>
