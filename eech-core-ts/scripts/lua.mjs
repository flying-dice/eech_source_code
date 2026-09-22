//
// Lua 5.1 helpers for the build gates.
//
//   node scripts/lua.mjs build <tsconfig>   run TSTL; any TSTL diagnostic fails the build
//   node scripts/lua.mjs run <file.lua>     execute with a Lua 5.1 interpreter
//
// The interpreter is $LUA, else the first of lua5.1 / lua / luajit whose
// _VERSION is "Lua 5.1" (DCS World embeds Lua 5.1). Missing Lua 5.1 fails the
// gate: generated Lua is never considered tested without executing it.
//

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = join(dirname(fileURLToPath(import.meta.url)), "..");

export function findLua51() {
	const candidates = process.env.LUA ? [process.env.LUA] : ["lua5.1", "lua", "luajit"];
	for (const candidate of candidates) {
		const probe = spawnSync(candidate, ["-e", "io.write(_VERSION)"], { encoding: "utf8" });
		if (probe.status === 0 && probe.stdout === "Lua 5.1") {
			return candidate;
		}
	}
	throw new Error(`no Lua 5.1 interpreter found (tried ${candidates.join(", ")}); install lua5.1 or set LUA`);
}

function build(tsconfig) {
	const tstl = join(projectRoot, "node_modules", ".bin", "tstl");
	const result = spawnSync(tstl, ["-p", tsconfig], { cwd: projectRoot, encoding: "utf8" });
	const output = `${result.stdout}${result.stderr}`;
	process.stdout.write(output);
	if (result.status !== 0 || /\b(warning|error) TSTL\b/.test(output)) {
		console.error(`TSTL build of ${tsconfig} reported diagnostics`);
		process.exit(1);
	}
}

function run(file) {
	const lua = findLua51();
	const result = spawnSync(lua, [file], { cwd: projectRoot, stdio: "inherit" });
	process.exit(result.status ?? 1);
}

const [command, arg] = process.argv.slice(2);

try {
	if (command === "build") {
		build(arg);
	} else if (command === "run") {
		run(arg);
	} else {
		console.error("usage: lua.mjs build <tsconfig> | run <file.lua>");
		process.exit(2);
	}
} catch (e) {
	console.error(e.message);
	process.exit(1);
}
