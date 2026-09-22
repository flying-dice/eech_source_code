#!/usr/bin/env node
//
// Builds the C reference harness: extracts the original EECH functions and
// compiles them with the shim. Fails (exit 1) when no C compiler is available,
// so the C reference gate can never pass vacuously.
//

import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { generateExtracted } from "./extract.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const projectRoot = join(here, "..");
const outDir = join(projectRoot, "build", "c-reference");

export const HARNESS_BINARY = join(outDir, "harness");

export function buildHarness() {
	mkdirSync(outDir, { recursive: true });
	writeFileSync(join(outDir, "eech_extracted.c"), generateExtracted());

	const cc = process.env.CC || "cc";
	const args = [
		"-std=c99",
		"-O0",
		"-Wall",
		"-Werror",
		"-Wno-unused-function",
		"-Wno-unused-variable",
		"-Wno-unused-but-set-variable",
		"-ffp-contract=off",
		"-fno-fast-math",
		"-I",
		outDir,
		"-I",
		here,
		join(here, "harness.c"),
		"-lm",
		"-o",
		HARNESS_BINARY,
	];
	const result = spawnSync(cc, args, { encoding: "utf8" });
	if (result.error || result.status !== 0) {
		throw new Error(`C reference harness build failed (${cc} ${args.join(" ")}):\n${result.error ?? ""}${result.stdout}${result.stderr}`);
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
