import { defineConfig } from "vitest/config";

// INVESTIGATION ONLY (issue #7): the FPU fidelity spike. Not part of `verify`.
export default defineConfig({
	test: {
		include: ["test/fpu-spike/**/*.fpu.test.ts"],
		globalSetup: ["test/fpu-spike/global-setup.ts"],
		testTimeout: 1_800_000,
		hookTimeout: 1_800_000,
	},
});
