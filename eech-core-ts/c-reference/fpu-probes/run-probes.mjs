#!/usr/bin/env node
//
// INVESTIGATION ONLY (issue #7), Investigation 6: builds probes.c for SSE and
// x87 (-O0 and -O2), runs each probe under every control word, and prints a
// markdown table plus the assembly of each probe function.
//
//   node c-reference/fpu-probes/run-probes.mjs [--asm]
//

import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const out = join(here, "..", "..", "build", "fpu-probes");
mkdirSync(out, { recursive: true });

const cc = process.env.CC || "cc";
const BUILDS = {
	sse: ["-msse2", "-mfpmath=sse", "-O0"],
	x87: ["-mfpmath=387", "-fexcess-precision=standard", "-O0"],
	"x87-O2": ["-mfpmath=387", "-fexcess-precision=standard", "-O2"],
};

for (const [name, flags] of Object.entries(BUILDS)) {
	const args = ["-std=gnu99", "-m32", ...flags, "-ffp-contract=off", join(here, "probes.c"), "-lm", "-o", join(out, `probes-${name}`)];
	const r = spawnSync(cc, args, { encoding: "utf8" });
	if (r.status !== 0) throw new Error(r.stderr);
	spawnSync(cc, ["-std=gnu99", "-m32", ...flags, "-ffp-contract=off", "-S", join(here, "probes.c"), "-o", join(out, `probes-${name}.s`)]);
}

// [label, build, x87 control word, mxcsr rounding]
const MODES = [
	["sse RN", "sse", "037f", 0],
	["sse RTZ", "sse", "0f7f", 3],
	["x87 RN PC64", "x87", "037f", 0],
	["x87 RN PC53", "x87", "027f", 0],
	["x87 RTZ PC53", "x87", "0e7f", 0],
	["x87 RTZ PC53 -O2", "x87-O2", "0e7f", 0],
	["x87 RN PC24", "x87", "007f", 0],
	["x87 RTZ PC24", "x87", "0c7f", 0],
];

const PROBES = [
	["timer", "1.0", "0.1", "20"],
	["timer", "0.7", "0.1", "20"],
	["timer", "5.0", "0.3", "30"],
	["timer", "1.0", "0.01", "200"],
	["subdivide", "0.1", "10"],
	["subdivide", "0.3", "10"],
	["subdivide", "0.7", "10"],
	["subdivide", "0.617", "2"],
	["subdivide", "0.29999998", "10"],
	["supply", "0.00001", "100"],
	["supply", "33.333", "100"],
	["supply", "12.838", "8.646"],
	["supply", "99.99999", "100"],
	["range", "1000.3", "2000.7", "10.1", "-3.3"],
	["range", "-7027.003", "362.58", "1414.948", "-6811.288"],
	["range", "12345.678", "-9876.543", "1.1", "2.2"],
	["map", "64", "4096"],
	["map", "255", "65795"],
	["map", "3", "5592406"],
	["sector", "511.5", "512"],
	["sector", "511.75", "512"],
	["sector", "1023.5", "512"],
	["sector", "512.5", "512"],
	["gnuc-fistp", "70000.7"],
];

const rows = [["probe", ...MODES.map((m) => m[0])]];
for (const p of PROBES) {
	const row = [p.join(" ")];
	for (const [, build, cw, rc] of MODES) {
		const r = spawnSync(join(out, `probes-${build}`), [cw, String(rc), ...p], { encoding: "utf8" });
		row.push(r.status === 0 ? r.stdout.trim() : `exit ${r.status}`);
	}
	rows.push(row);
}

console.log(rows.map((r, i) => `| ${r.join(" | ")} |${i === 0 ? `\n|${r.map(() => "---").join("|")}|` : ""}`).join("\n"));

if (process.argv.includes("--asm")) {
	for (const name of Object.keys(BUILDS)) {
		const s = readFileSync(join(out, `probes-${name}.s`), "utf8");
		for (const fn of ["probe_timer", "probe_subdivide", "probe_supply", "get_approx_2d_range", "probe_map"]) {
			const start = s.indexOf(`${fn}:`);
			const end = s.indexOf(".size", start);
			console.log(`\n### ${name} ${fn}\n\`\`\`\n${s.slice(start, end).split("\n").filter((l) => !/^\s*\.(cfi|loc)/.test(l)).join("\n")}\`\`\``);
		}
	}
}
