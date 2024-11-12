import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
    plugins: [sveltekit()],
    optimizeDeps: {
        exclude: ["phosphor-svelte"],
    },
    assetsInclude: ["static/**/*"],
    // https://stackoverflow.com/questions/78997907/the-legacy-js-api-is-deprecated-and-will-be-removed-in-dart-sass-2-0-0
    css: {
        preprocessorOptions: {
            scss: {
                api: "modern-compiler",
            },
        },
    },
	server: {
		fs: {
			allow: ["assets"]
		}
	}
});
