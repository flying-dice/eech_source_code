import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		include: ["test/**/*.test.ts"],
		exclude: ["test/c-reference/**"],
		coverage: {
			provider: "istanbul",
			include: ["src/**/*.ts"],
			// Type-only modules emit no JavaScript; nothing else is excluded.
			// Individually justified exclusions live in the source as
			// `istanbul ignore` comments and are listed in docs/port-manifest.md.
			exclude: ["src/ports/**"],
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
