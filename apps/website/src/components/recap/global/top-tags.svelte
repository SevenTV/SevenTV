<script lang="ts">
	const tags = [
		{ number: 1, text: "#cat", textColor: "#191825", bgColor: "#FF3F7F" },
		{ number: 2, text: "#christmas", textColor: "#191825", bgColor: "#a6ff96" },
		{ number: 3, text: "#cute", textColor: "#191825", bgColor: "#FFC400" },
		{ number: 4, text: "#peepo", textColor: "#191825", bgColor: "#FFFFFF" },
		{ number: 5, text: "#dance", textColor: "#191825", bgColor: "#8C00FF" },
		{ number: 6, text: "#halloween", textColor: "#191825", bgColor: "#15F5BA" },
		{ number: 7, text: "#forsen", textColor: "#191825", bgColor: "#FF67E7" },
		{ number: 8, text: "#meow", textColor: "#191825", bgColor: "#836FFF" },
		{ number: 9, text: "#buh", textColor: "#191825", bgColor: "#E9B3FB" },
		{ number: 10, text: "#xqc", textColor: "#191825", bgColor: "#8C00FF" },
	];
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div>
				<p class="recap title">MOST POPULAR</p>
				<p class="recap title2">#TAGS</p>
			</div>
			<div>
				{#each tags as tag (tag.number)}
					<p class="recap title3" style="color: #FF3F7F; margin: 2rem;">
						{tag.number}.
						<span
							class="recap title3"
							style="color: {tag.textColor}; background: {tag.bgColor}; padding: 0.4rem;"
						>
							<a target="_blank" style={`color: ${tag.textColor}`} href={`https://7tv.app/emotes?t=${tag.text.replace("#", "")}`}>{tag.text}</a>
						</span>
					</p>
				{/each}
			</div>
			<div>
				<p class="recap subtitle">These are the most popular</p>
				<p class="recap subtitle">themes of the emotes uploaded!</p>
			</div>
		</div>
	</header>
</section>

<style lang="scss">
    .recap {
        font-size: 5rem;
        margin: 0;
        font-family: "BBH Bartleby", sans-serif;
        font-weight: 200;
        text-align: center;

        &.title {
            font-size: 2rem;
        }

        &.title2 {
            font-size: 5rem;
        }
        &.title3 {
            font-size: 2rem;
            text-align: center;
        }

        &.subtitle {
            font-size: 1.5rem;
            font-weight: 1000;
            text-align: center;
            font-family: "Syne Bold", sans-serif;
            color: #ffffff;
        }
    }
    .side-by-side {
        display: flex;
        align-items: center;
        gap: 10rem;
    }

    .hero {
        position: relative;
        min-height: 40rem;
        color: #fff;
        background-color: #191825;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .top {
        margin-top: 5rem;
        margin-bottom: 5rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-left: 3rem;

        .header-content {
            font-weight: 900;
            font-size: 2rem;
            color: #ffffff;
        }
    }

    @media (max-width: 1500px) {
        .side-by-side {
            flex-direction: column;
            gap: 2rem;
        }
        .hero {
            min-height: unset;
            padding: 2rem 0;
        }
        .top {
            margin-top: 2rem;
            margin-bottom: 2rem;
            margin-left: 0;
        }
        .recap {
            font-size: 2.5rem;

            &.title {
                font-size: 1.2rem;
            }
            &.title2 {
                font-size: 2.5rem;
            }
            &.title3 {
                font-size: 1.2rem;
            }
            &.subtitle {
                font-size: 1rem;
            }
        }
    }

    @media (max-width: 1500px) {
        .hero {
            padding: 1rem 0.5rem;
        }
        .recap {
            font-size: 1.5rem;

            &.title {
                font-size: 1rem;
            }
            &.title2 {
                font-size: 1.5rem;
            }
            &.title3 {
                font-size: 1rem;
            }
            &.subtitle {
                font-size: 0.9rem;
            }
        }
        .side-by-side {
            gap: 1rem;
        }
    }
</style>
