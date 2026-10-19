<script lang="ts">
	import { onMount } from "svelte";

	type Particle = {
		id: number;
		x: number;
		y: number;
		size: number;
		life: number;
		speedY: number;
		drift: number;
	};

	let particles = $state<Particle[]>([]);
	let mouseX = $state(0);
	let mouseY = $state(0);

	let idCounter = 0;
	const MAX_PARTICLES = 60;

    let lastSpawn = 0;

function handleMouseMove(e: MouseEvent) {
	mouseX = e.clientX;
	mouseY = e.clientY;

	const now = performance.now();

	// spawn at most every 40ms
	if (now - lastSpawn > 40) {
		spawnParticle(mouseX, mouseY);
		lastSpawn = now;
	}
}

	function spawnParticle(x: number, y: number) {
		if (particles.length > MAX_PARTICLES) return;

		particles.push({
			id: idCounter++,
			x,
			y,
			size: Math.random() * 8 + 6,
			life: 1,
			speedY: Math.random() * 0.8 + 0.5,
			drift: (Math.random() - 0.5) * 1.2
		});
	}

	function update() {
		for (let p of particles) {
			p.y -= p.speedY;
			p.x += p.drift;
			p.life -= 0.03;
		}

		// remove dead particles
		particles = particles.filter(p => p.life > 0);

		requestAnimationFrame(update);
	}

	onMount(() => {
	window.addEventListener("mousemove", handleMouseMove);
	requestAnimationFrame(update);

	return () => {
		window.removeEventListener("mousemove", handleMouseMove);
	};
});
</script>

<div class="container">
	{#each particles as p (p.id)}
		<!-- svelte-ignore element_invalid_self_closing_tag -->
		<div
			class="heart"
			style="
				left: {p.x}px;
				top: {p.y}px;
				width: {p.size}px;
				height: {p.size}px;
				opacity: {p.life};
			"
		/>
	{/each}
</div>

<style>
.container {
	position: fixed;
	inset: 0;
	overflow: hidden;
	pointer-events: none; /* ✅ THIS FIXES IT */
	z-index: 9999;
}


.heart {
	position: absolute;
	transform: rotate(-45deg);
	background: #ff6b9a;
	pointer-events: none; /* extra safety */
}

	.heart::before,
	.heart::after {
		content: "";
		position: absolute;
		width: 100%;
		height: 100%;
		background: inherit;
		border-radius: 50%;
	}

	.heart::before {
		top: -50%;
		left: 0;
	}

	.heart::after {
		top: 0;
		right: -50%;
	}
</style>
