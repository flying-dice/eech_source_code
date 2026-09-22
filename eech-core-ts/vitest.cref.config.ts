import { defineConfig } from "vitest/config";

// C reference differential tests: require a C compiler (see c-reference/build.mjs).
export default defineConfig({
	test: {
		include: ["test/c-reference/**/*.cref.test.ts"],
		globalSetup: ["test/c-reference/global-setup.ts"],
		testTimeout: 120_000,
	},
});
