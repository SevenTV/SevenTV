import adapter from "@sveltejs/adapter-static";
import { phosphorSvelteOptimize } from "phosphor-svelte/preprocessor";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://kit.svelte.dev/docs/integrations#preprocessors
	// for more information about preprocessors
	preprocess: [phosphorSvelteOptimize(), vitePreprocess()],

	kit: {
		// See https://kit.svelte.dev/docs/adapters for more information about adapters.
		adapter: adapter({
			pages: "build",
			assets: "build",
			// SPA index file
			fallback: "spa.html",
			strict: true,
			precompress: true,
		}),
		alias: {
			$: "./src",
			$assets: "./assets",
		},
	},
};

export default config;
