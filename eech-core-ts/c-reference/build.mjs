#!/usr/bin/env node
//
// Builds the C reference harness: generates project.h and the verbatim
// extracts, then compiles the harness, the extracts and the original EECH
// translation units listed in extract.mjs. Fails (exit 1) when no C compiler
// is available, so the C reference gate can never pass vacuously.
//

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { AP_SOURCE, REAL_TRANSLATION_UNITS, writeGenerated } from "./extract.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(here, "..");
const repoRoot = join(projectRoot, "..");
const outDir = join(projectRoot, "build", "c-reference");

export const HARNESS_BINARY = join(outDir, "harness");

const COMMON = [
	"-std=gnu99",
	"-O0",
	"-ffp-contract=off",
	"-fno-fast-math",
	// build/c-reference first, so the original files' #include "project.h" finds the reduced one
	"-I",
	outDir,
	"-I",
	here,
	"-I",
	join(repoRoot, AP_SOURCE),
];

// Our own sources: every warning is an error.
const OWN_FLAGS = ["-Wall", "-Werror", "-Wno-unused-function", "-Wno-unused-variable", "-Wno-unused-but-set-variable"];

// Original EECH sources: their historical warnings are not ours to fix, but the
// ones that would hide a mismatch with the harness environment are errors.
const ORIGINAL_FLAGS = ["-Werror=implicit-function-declaration", "-Werror=incompatible-pointer-types", "-Werror=int-conversion", "-Werror=return-type", "-w"];

function compile(cc, source, flags) {
	const object = join(outDir, `${source.replace(/[\\/]/g, "_")}.o`);
	const args = [...COMMON, ...flags, "-c", source, "-o", object];
	const result = spawnSync(cc, args, { encoding: "utf8" });
	if (result.error || result.status !== 0) {
		throw new Error(`C reference harness build failed (${cc} ${args.join(" ")}):\n${result.error ?? ""}${result.stdout}${result.stderr}`);
	}
	return object;
}

export function buildHarness() {
	writeGenerated(outDir);

	const cc = process.env.CC || "cc";
	const objects = [
		compile(cc, join(here, "harness.c"), OWN_FLAGS),
		// verbatim original code: original-code flags
		compile(cc, join(outDir, "eech_extracted.c"), ORIGINAL_FLAGS),
		...REAL_TRANSLATION_UNITS.map((unit) => compile(cc, join(repoRoot, unit), ORIGINAL_FLAGS)),
	];

	const link = spawnSync(cc, [...objects, "-lm", "-o", HARNESS_BINARY], { encoding: "utf8" });
	if (link.error || link.status !== 0) {
		throw new Error(`C reference harness link failed:\n${link.error ?? ""}${link.stdout}${link.stderr}`);
	}
	return HARNESS_BINARY;
}

const isMain = process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1];

if (isMain) {
	try {
		console.log(`built ${buildHarness()}`);
	} catch (e) {
		console.error(e.message);
		process.exit(1);
	}
}
