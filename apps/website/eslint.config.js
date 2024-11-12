import globals from "globals";
import pluginJs from "@eslint/js";
import sveltePlugin from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";
import tseslint from "typescript-eslint";

export default tseslint.config(
	{
		ignores: [
			"DS_Store",
			"node_modules",
			"build",
			"dist",
			".svelte-kit",
			"package",
			".env",
			".env.*",
			"pnpm-lock.yaml",
			"src/gql/*",
		],
	},
	{
		rules: {
			"@typescript-eslint/no-unused-vars": [
				"error",
				{
					argsIgnorePattern: "^_",
					varsIgnorePattern: "^_",
					caughtErrorsIgnorePattern: "^_",
				},
			],
		},
	},
	{ languageOptions: { globals: { NodeJS: true, ...globals.browser } } },
	pluginJs.configs.recommended,
	...tseslint.configs.recommended,
	...sveltePlugin.configs["flat/recommended"],
	{
		files: ["**/*.svelte", "*.svelte"],
		languageOptions: {
			parser: svelteParser,
			parserOptions: {
				parser: {
					ts: "@typescript-eslint/parser",
				},
				extraFileExtensions: [".svelte"],
			},
		},
	},
	{
		files: ["**/*.svelte.ts", "*.svelte.ts"],
		languageOptions: {
			parser: svelteParser,
			parserOptions: {
				parser: tseslint.parser,
			},
		},
	},
	{
		files: ["**/*.svelte.js", "*.svelte.js"],
		languageOptions: {
			parser: svelteParser,
		},
	},
);
