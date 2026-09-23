//
// INVESTIGATION ONLY (issue #7): runs scenario inputs through the canonical C
// oracle and every floating-point variant, and records where they differ.
//

import { spawnSync } from "node:child_process";
import { join } from "node:path";
// @ts-expect-error plain ESM build script without type declarations
import { HARNESS_BINARY } from "../../c-reference/build.mjs";
// @ts-expect-error plain ESM module without type declarations
import { FPU_VARIANTS as VARIANTS } from "../../c-reference/fpu-variants.mjs";

export interface Variant {
	name: string;
	model: string;
	flags: string[];
	defines: string[];
}

export const FPU_VARIANTS = VARIANTS as Variant[];

export function variantBinary(name: string): string {
	return join(HARNESS_BINARY as string, "..", "..", "c-reference-fpu", name, "harness");
}

export interface Run {
	status: number | null;
	stdout: string;
	stderr: string;
}

export function runBinary(binary: string, input: string): Run {
	const run = spawnSync(binary, [], { input, encoding: "utf8" });
	if (run.error) {
		throw run.error;
	}
	if (run.status === 4) {
		throw new Error(`${binary}: ${run.stderr}`);
	}
	return { status: run.status, stdout: run.stdout, stderr: run.stderr };
}

export function runCanonical(input: string): Run {
	return runBinary(HARNESS_BINARY as string, input);
}

export interface Difference {
	index: number;
	input: string;
	canonical: string[];
	variant: string[];
}

// The lines that differ, in order (a changed line appears as a -/+ pair).
export function lineDiff(a: string[], b: string[]): string[] {
	const out: string[] = [];
	const n = Math.max(a.length, b.length);
	for (let i = 0; i < n; i++) {
		if (a[i] !== b[i]) {
			if (a[i] !== undefined) out.push(`- ${a[i]}`);
			if (b[i] !== undefined) out.push(`+ ${b[i]}`);
		}
	}
	return out;
}

// the kind of output line (its first word), as the outcome field it reports
export function lineKind(line: string): string {
	return line.replace(/^[-+] /, "").split(" ")[0];
}

export interface CorpusResult {
	corpus: string;
	count: number;
	byVariant: Record<string, { differing: number; kinds: Record<string, number>; examples: Difference[] }>;
}

export function differential(corpus: string, inputs: string[], exampleLimit = 3): CorpusResult {
	const result: CorpusResult = { corpus, count: inputs.length, byVariant: {} };
	const canonical = inputs.map((input) => runCanonical(input));

	for (const variant of FPU_VARIANTS) {
		const entry = { differing: 0, kinds: {} as Record<string, number>, examples: [] as Difference[] };
		inputs.forEach((input, index) => {
			const run = runBinary(variantBinary(variant.name), input);
			const c = canonical[index];
			if (run.stdout === c.stdout && run.status === c.status) {
				return;
			}
			entry.differing += 1;
			const cl = [...c.stdout.split("\n"), `exit ${c.status}`];
			const vl = [...run.stdout.split("\n"), `exit ${run.status}`];
			const kinds = new Set(lineDiff(cl, vl).map(lineKind));
			for (const k of kinds) {
				entry.kinds[k] = (entry.kinds[k] ?? 0) + 1;
			}
			if (entry.examples.length < exampleLimit) {
				entry.examples.push({ index, input, canonical: cl, variant: vl });
			}
		});
		result.byVariant[variant.name] = entry;
	}
	return result;
}
