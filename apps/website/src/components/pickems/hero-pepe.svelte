<script lang="ts">
	import pickemsFace from "$assets/pickems_peepo_no_eyes.svg?url";
	import pickemsHand from "$assets/pickems_peepo_hand.png?url";
	import pickemsEyeLeft from "$assets/pickems_peepo_eye.svg?url";
	import pickemsEyeRight from "$assets/pickems_peepo_eye.svg?url";
</script>

<div>
	<img
		src={pickemsEyeRight}
		style="
		z-index: 1;
		width: 2.5rem;
		bottom: 4rem;
		left: 6rem;"
		alt="Pepe Right Eye"
	/>
	<img
		src={pickemsEyeLeft}
		style="
		z-index: 1;
		width: 2.5rem;
		left: 10.5rem;
		bottom: 4rem;"
		alt="Pepe Left Eye"
	/>
	<img
		src={pickemsHand}
		style="
		width: 4rem;
		left: 15.5rem;
		z-index: 3;
        top: 1rem;"
		alt="Pepe Hand"
	/>
	<img
		src={pickemsFace}
		style="bottom: 1rem;
        width: 11rem;
        left: 3rem;"
		alt="Pepe Face"
	/>
</div>

<style lang="scss">
	img {
		position: absolute;
	}
</style>
