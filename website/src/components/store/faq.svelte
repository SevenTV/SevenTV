<script lang="ts">
	import { Plus, Question } from "phosphor-svelte";

	const qas = [
		{
			q: "Lorem ipsum dolor sit, amet consectetur adipisicing elit?",
			a: "Lorem ipsum dolor, sit amet consectetur adipisicing elit. Ipsum, velit, obcaecati nulla assumenda est dicta praesentium excepturi magnam quibusdam libero minus ratione quod architecto placeat repellat, atque molestias! Quos, nobis.",
		},
		{
			q: "Lorem ipsum dolor sit, amet consectetur adipisicing elit?",
			a: "Lorem ipsum dolor, sit amet consectetur adipisicing elit. Ipsum, velit, obcaecati nulla assumenda est dicta praesentium excepturi magnam quibusdam libero minus ratione quod architecto placeat repellat, atque molestias! Quos, nobis.",
		},
		{
			q: "Lorem ipsum dolor sit, amet consectetur adipisicing elit?",
			a: "Lorem ipsum dolor, sit amet consectetur adipisicing elit. Ipsum, velit, obcaecati nulla assumenda est dicta praesentium excepturi magnam quibusdam libero minus ratione quod architecto placeat repellat, atque molestias! Quos, nobis.",
		},
	];
</script>

<section>
	<div class="header">
		<Question />
		<h2>Frequently Asked Questions</h2>
	</div>
	<div class="faq">
		{#each qas as qa}
			<hr />
			<label class="qa">
				<div class="q">
					{qa.q}
					<Plus />
				</div>

				<!-- Pseudo checkbox box for not having to use js for expanding -->
				<input type="checkbox" name="question" hidden />
				<span class="a">{qa.a}</span>
			</label>
		{/each}
	</div>
</section>

<style lang="scss">
	section {
		flex-basis: 0;
		flex-grow: 1;

		background-color: var(--bg-medium);
		border-radius: 0.5rem;
		padding: 1rem 1.25rem;
		min-width: 12rem;
	}

	.header {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	h2 {
		font-size: 1rem;
		font-weight: 600;
	}

	.faq {
		margin-top: 1rem;

		display: flex;
		flex-direction: column;
	}

	.a {
		margin-top: 0.5rem;
		color: var(--text-light);

		display: none;
	}

	.qa {
		font-size: 0.875rem;
		padding: 1rem 0;
		cursor: pointer;
		user-select: none;

		& > .q {
			display: flex;
			justify-content: space-between;
			align-items: center;
		}

		&:hover,
		&:focus-visible {
			background-color: var(--bg-light);
		}

		& > input:checked + .a {
			display: inline-block;
		}
	}
</style>
