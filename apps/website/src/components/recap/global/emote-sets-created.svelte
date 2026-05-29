<script lang="ts">
</script>

<section class="hero">
	<header class="top">
		<div class="header-content side-by-side">
			<div>
				<img src="../../../assets/recap-star-icon.png" alt="Emote Sets Created" style="height: 20rem; margin-bottom: 1rem;" />
			</div>
			<div>
                <p class="recap subtitle">You really love sets don't you...</p>
				<p class="recap title">232,895</p>
				<p class="recap title emotes">EMOTE SETS</p>
				<p class="recap title created">CREATED</p>
			</div>
		</div>
	</header>
</section>

<style lang="scss">
    .recap {
        font-size: 7rem;
        margin: 0;
        font-family: "BBH Bartleby", sans-serif;
        font-weight: 200;

        &.title {
            color: #A6FF96;
            &.emotes {
                color: #ffffff;
                font-weight: 100;
                font-size: 5rem;
                -webkit-text-stroke: 3px #1D1D1D;
            }
            &.created {
                font-size: 5rem;
                color: transparent;
                -webkit-text-stroke: 3px #ffffff;
            }
        }

        &.subtitle {
            font-size: 2.5rem;
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
        background-color: #1d1d1d;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .top {
        margin-top: 10rem;
        display: flex;
        flex-direction: column;
        align-items: center;
        margin-left: 3rem;

        .header-content {
            font-weight: 900;
            font-size: 2rem;
            color: #a970ff;
        }
    }

    // Responsive styles
    @media (max-width: 1500px) {
        .recap {
            font-size: 4.5rem;
            &.title.emotes,
            &.title.created {
                font-size: 3rem;
            }
            &.subtitle {
                font-size: 1.8rem;
            }
        }
        .side-by-side {
            gap: 4rem;
        }
        .hero {
            min-height: 28rem;
        }
        .top {
            margin-top: 5rem;
            margin-left: 1rem;
            margin-bottom: 1rem;
        }
        .header-content img {
            height: 12rem !important;
        }
    }

    @media (max-width: 900px) {
        .side-by-side {
            flex-direction: column;
            gap: 2rem;
        }
        .header-content img {
            height: 8rem !important;
            margin-bottom: 0.5rem;
        }
        .recap {
            font-size: 2.5rem;
            &.title.emotes,
            &.title.created {
                font-size: 1.5rem;
            }
            &.subtitle {
                font-size: 1.2rem;
            }
        }
        .hero {
            min-height: 18rem;
        }
        .top {
            margin-top: 2rem;
            margin-left: 0;
        }
    }
</style>
