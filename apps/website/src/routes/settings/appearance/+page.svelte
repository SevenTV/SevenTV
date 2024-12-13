<script lang="ts">
	import Select from "$/components/input/select.svelte";
	import { Faders, Moon, PersonSimpleCircle, PersonSimpleRun, Sun } from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { reducedMotion, theme } from "$/lib/layout";
	import { prefersReducedMotion } from "svelte/motion";
	import { MediaQuery } from "svelte/reactivity";

	let systemTheme = $derived(
		new MediaQuery("(prefers-color-scheme: light)").current ? "Light" : "Dark",
	);
</script>

<svelte:head>
	<title>Appearance - {$t("page_titles.suffix")}</title>
</svelte:head>

<section>
	<div>
		<h2>Appearance</h2>
		<span class="details">Change the look and feel</span>
	</div>
	<div class="content">
		<div class="setting">
			{#if $theme === "system-theme"}
				<h3>Theme - {systemTheme}</h3>
			{:else}
				<h3>Theme</h3>
			{/if}
			{#snippet system()}
				<Faders />
			{/snippet}
			{#snippet dark()}
				<Moon />
			{/snippet}
			{#snippet light()}
				<Sun />
			{/snippet}
			<Select
				options={[
					{ value: "system-theme", label: "System", icon: system },
					{ value: "dark-theme", label: "Dark", icon: dark },
					{ value: "light-theme", label: "Light", icon: light },
				]}
				bind:selected={$theme}
			/>
		</div>
		<hr />
		<div class="setting">
			{#if $reducedMotion === "reduced-motion-system"}
				<h3>Reduced Motion - {prefersReducedMotion.current ? "Enabled" : "Disabled"}</h3>
			{:else}
				<h3>Reduced Motion</h3>
			{/if}
			{#snippet system()}
				<Faders />
			{/snippet}
			{#snippet enabled()}
				<PersonSimpleCircle />
			{/snippet}
			{#snippet disabled()}
				<PersonSimpleRun />
			{/snippet}
			<Select
				options={[
					{ value: "reduced-motion-system", label: "System", icon: system },
					{ value: "reduced-motion-enabled", label: "Enabled", icon: enabled },
					{ value: "reduced-motion-disabled", label: "Disabled", icon: disabled },
				]}
				bind:selected={$reducedMotion}
			/>
		</div>
	</div>
</section>

<style lang="scss">
	@use "../../../styles/settings.scss";

	h3 {
		font-size: 0.875rem;
		font-weight: 500;
	}

	.setting {
		display: flex;
		flex-direction: column;
		align-items: start;
		gap: 0.5rem;
	}
</style>
