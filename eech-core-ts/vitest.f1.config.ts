import { defineConfig } from "vitest/config";

// INVESTIGATION ONLY (issue #14, F1): the uninitialised route height probe. Not part of `verify`.
export default defineConfig({
	test: {
		include: ["test/f1-probe/**/*.probe.test.ts"],
		globalSetup: ["test/f1-probe/global-setup.ts"],
		testTimeout: 600_000,
		hookTimeout: 1_800_000,
	},
});
