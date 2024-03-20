<script lang="ts">
	import Button from "$/components/input/button.svelte";
	import HideOn from "$/components/hide-on.svelte";
	import PaintPreview from "$/components/paint-preview.svelte";
	import Select from "$/components/input/select.svelte";
	import Banner from "$/components/store/banner.svelte";
	import { Gift, MagnifyingGlass, ShoppingCartSimple } from "phosphor-svelte";
	import { priceFormat } from "$/lib/utils";
	import TextInput from "$/components/input/text-input.svelte";
	import { t } from "svelte-i18n";

	const bundles = [
		{
			name: "Summer Bundle",
			price: 199,
			new: true,
		},
		{
			name: "Winter Bundle",
			price: 500,
			new: false,
		},
		{
			name: "Halloween Bundle",
			price: 299,
			new: false,
		},
		{
			name: "Loser Bundle",
			price: 1,
			new: false,
		},
		{
			name: "Random Bundle",
			price: 199,
			new: false,
		},
		{
			name: "Forsen Bundle",
			price: 1099,
			new: false,
			oldPrice: 10099,
		},
	];
</script>

<svelte:head>
	<title>{$t("common.paint_bundles")} - {$t("page_titles.suffix")}</title>
</svelte:head>

<Banner
	title={$t("pages.store.paint_bundles.banner_title")}
	subtitle={$t("pages.store.paint_bundles.banner_subtitle")}
	gradientColor="#27cfb1"
/>
<section>
	<div class="header">
		<h2>{$t("common.paint_bundles")}</h2>
		<div class="buttons">
			<HideOn mobile>
				<TextInput placeholder={$t("labels.search")} style="max-width: 12.5rem">
					<MagnifyingGlass slot="icon" />
				</TextInput>
			</HideOn>
			<Button hideOnDesktop>
				<MagnifyingGlass />
			</Button>
			<Select
				options={[
					{ value: "none", label: $t("labels.no_filters") },
					{ value: "filters", label: $t("labels.filters") },
				]}
			/>
		</div>
	</div>
	<div class="grid">
		{#each bundles as bundle}
			<div class="bundle">
				<div class="header">
					<span class="name">{bundle.name}</span>
					{#if bundle.new}
						<span class="new">{$t("common.new")}</span>
					{/if}
				</div>
				{#each Array(3) as _}
					<PaintPreview />
				{/each}
				<div class="buttons">
					<Button secondary>
						<Gift slot="icon" />
						{$t("labels.gift")}
					</Button>
					<Button secondary>
						<ShoppingCartSimple slot="icon" />
						<span>
							{#if bundle.oldPrice}
								<del>{priceFormat().format(bundle.oldPrice / 100)}</del>
							{/if}
							{priceFormat().format(bundle.price / 100)}
						</span>
					</Button>
				</div>
			</div>
		{/each}
	</div>
</section>

<style lang="scss">
	section {
		margin-top: 1rem;
		margin-inline: auto;
		max-width: 80rem;
	}

	h2 {
		font-size: 1.25rem;
		font-weight: 700;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.5rem;

		.buttons {
			display: flex;
			gap: 0.5rem;
			align-items: center;
		}
	}

	.grid {
		margin-top: 1rem;

		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
		gap: 1rem;
	}

	.bundle {
		padding: 1rem 1.25rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
		background-color: var(--bg-medium);
		border-radius: 0.5rem;

		.header {
			display: flex;
			justify-content: space-between;
			align-items: center;

			.name {
				font-weight: 500;
			}

			.new {
				color: var(--subscriber);
				border: 1px solid var(--subscriber);
				padding: 0.2rem 0.5rem;
				border-radius: 0.25rem;
				font-size: 0.625rem;
				text-transform: uppercase;
			}
		}

		.buttons {
			display: flex;
			gap: 0.5rem;
			justify-content: flex-end;
		}

		del {
			font-size: 0.75rem;
		}
	}
</style>
