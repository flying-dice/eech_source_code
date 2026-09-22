//
// Lint rules that protect behaviour after TSTL transpilation to Lua 5.1.
// This is not a style guide; formatting is left alone.
//

import tseslint from "typescript-eslint";

export default tseslint.config(
	{
		ignores: ["build/**", "node_modules/**", "src/generated/**", "test/scenarios/generated/**", "**/*.mjs", "*.config.ts"],
	},
	{
		files: ["src/**/*.ts", "test/**/*.ts"],
		languageOptions: {
			parser: tseslint.parser,
			parserOptions: {
				projectService: {
					allowDefaultProject: ["test/lua/*.ts"],
				},
				tsconfigRootDir: import.meta.dirname,
			},
		},
		plugins: { "@typescript-eslint": tseslint.plugin },
		rules: {
			// C tests ints for truth (`if (in_use)`); in Lua 0 and "" are TRUE.
			// Numbers and strings must be compared explicitly.
			"@typescript-eslint/strict-boolean-expressions": [
				"error",
				{ allowString: false, allowNumber: false, allowNullableObject: true, allowNullableBoolean: false, allowNullableNumber: false, allowNullableString: false, allowAny: false },
			],
			// `==` / `!=` have no Lua equivalent for null/undefined coercion
			eqeqeq: ["error", "always"],
		},
	},
	{
		files: ["src/**/*.ts"],
		rules: {
			// The campaign core is environment-free: no DCS, no Node, no host globals.
			"no-restricted-globals": [
				"error",
				...["env", "coalition", "world", "timer", "trigger", "land", "atmosphere", "missionCommands", "net", "Unit", "Group", "StaticObject", "Airbase", "Object", "Controller", "country", "AI", "radio", "Spot", "Weapon", "SceneryObject", "process", "require", "console", "globalThis", "window", "setTimeout", "setInterval", "Date"].map((name) => ({
					name,
					message: "campaign core must reach the environment through src/ports",
				})),
			],
			"no-restricted-imports": ["error", { patterns: ["node:*", "fs", "path", "child_process", "*/test/*", "../../test/*"] }],
			"no-restricted-properties": [
				"error",
				{ object: "Math", property: "random", message: "randomness must come through a port" },
				{ object: "Math", property: "fround", message: "not available in Lua 5.1; use toFloat32" },
				{ object: "Math", property: "log2", message: "not available in Lua 5.1" },
			],
		},
	},
);
