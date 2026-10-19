<script lang="ts">
	import Select from "$/components/input/select.svelte";
	import {
		Faders,
		Moon,
		PersonSimpleCircle,
		PersonSimpleRun,
		Sun,
		ArrowRight,
		ArrowLeft,
	} from "phosphor-svelte";
	import { t } from "svelte-i18n";
	import { reducedMotion, theme, pageRightToLeft } from "$/lib/layout";
	import { prefersReducedMotion } from "svelte/motion";
	import { MediaQuery } from "svelte/reactivity";

	let systemTheme = $derived(
		new MediaQuery("(prefers-color-scheme: light)").current
			? $t("pages.settings.appearance.theme.options.light")
			: $t("pages.settings.appearance.theme.options.dark"),
	);
</script>

<svelte:head>
	<title>{$t("pages.settings.appearance.title")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<section>
	<div>
		<h2>{$t("pages.settings.appearance.title")}</h2>
		<span class="details">{$t("pages.settings.appearance.details")}</span>
	</div>
	<div class="content">
		<div class="setting">
			{#if $theme === "system-theme"}
				<h3>{$t("pages.settings.appearance.theme.header")} - {systemTheme}</h3>
			{:else}
				<h3>{$t("pages.settings.appearance.theme.header")}</h3>
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
					{
						value: "system-theme",
						label: $t("pages.settings.appearance.theme.options.system"),
						icon: system,
					},
					{
						value: "dark-theme",
						label: $t("pages.settings.appearance.theme.options.dark"),
						icon: dark,
					},
					{
						value: "light-theme",
						label: $t("pages.settings.appearance.theme.options.light"),
						icon: light,
					},
				]}
				bind:selected={$theme}
			/>
		</div>
		<hr />
		<div class="setting">
			{#if $reducedMotion === "reduced-motion-system"}
				<h3>
					{$t("pages.settings.appearance.motion.header")} - {prefersReducedMotion.current
						? $t("pages.settings.appearance.motion.options.enabled")
						: $t("pages.settings.appearance.motion.options.disbaled")}
				</h3>
			{:else}
				<h3>{$t("pages.settings.appearance.motion.header")}</h3>
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
					{
						value: "reduced-motion-system",
						label: $t("pages.settings.appearance.motion.options.system"),
						icon: system,
					},
					{
						value: "reduced-motion-enabled",
						label: $t("pages.settings.appearance.motion.options.enabled"),
						icon: enabled,
					},
					{
						value: "reduced-motion-disabled",
						label: $t("pages.settings.appearance.motion.options.disbaled"),
						icon: disabled,
					},
				]}
				bind:selected={$reducedMotion}
			/>
		</div>
		<hr />
		<div class="setting">
			<h3>{$t("pages.settings.appearance.right-to-left-layout.header")}</h3>
			{#snippet enabled()}
				<ArrowLeft />
			{/snippet}
			{#snippet disabled()}
				<ArrowRight />
			{/snippet}
			<Select
				options={[
					{
						value: "rtl",
						label: $t("pages.settings.appearance.right-to-left-layout.options.rtl"),
						icon: enabled,
					},
					{
						value: "ltr",
						label: $t("pages.settings.appearance.right-to-left-layout.options.ltr"),
						icon: disabled,
					},
				]}
				bind:selected={$pageRightToLeft}
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
