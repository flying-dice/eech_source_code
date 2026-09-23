import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		include: ["test/**/*.test.ts"],
		// the FPU spike (issue #7) is investigation only: npm run spike:fpu
		exclude: ["test/c-reference/**", "test/fpu-spike/**"],
		coverage: {
			provider: "istanbul",
			include: ["src/**/*.ts"],
			// Nothing is excluded. Type-only modules (src/ports) emit no
			// statements and are reported as fully covered; if one starts
			// emitting runtime code it is measured like everything else.
			// Individually justified exclusions live in the source as
			// `istanbul ignore` comments and are listed in docs/port-manifest.md.
			reporter: ["text", "json-summary", "html"],
			reportsDirectory: "build/coverage",
			thresholds: {
				statements: 100,
				branches: 100,
				functions: 100,
				lines: 100,
			},
		},
	},
});
