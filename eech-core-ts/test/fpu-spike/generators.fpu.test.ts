//
// INVESTIGATION ONLY (issue #7), Investigation 5: the existing generators
// (the same seeds and counts as test/c-reference/differential.cref.test.ts)
// through the canonical oracle and every floating-point variant.
//
// TypeScript equals the canonical oracle on these corpora (the frozen
// differential tests), so a variant difference is a TS-vs-variant difference.
// Since the RTZ migration the canonical oracle rounds toward zero; `sse-rn` is
// the former canonical oracle.
//
// Writes build/fpu-spike/generators.json; asserts only that the control
// variant (canonical flags through the variant path) equals canonical.
//

import { mkdirSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { formatNumberForC } from "../c-reference/c-harness";
import { generateRandomLifecycles, generateRandomScenarios, generateRandomTimelines } from "../c-reference/random-scenarios";
import { serialiseScenario } from "../scenarios/campaign-scenario";
import { serialiseLifecycle } from "../scenarios/lifecycle-scenario";
import { serialiseTimeline } from "../scenarios/update-timeline";
import { differential, type CorpusResult } from "./run";

describe("FPU spike: generator corpora", () => {
	it("runs every corpus through every variant", () => {
		const results: CorpusResult[] = [
			differential("slice1-scenarios", generateRandomScenarios(0xeec4, 1500).map((s) => serialiseScenario(s, formatNumberForC))),
			differential("slice2-timelines", generateRandomTimelines(0x7153, 1000).map((s) => serialiseTimeline(s, formatNumberForC))),
			differential("slice3-lifecycles", generateRandomLifecycles(0x11fe, 1000).map((s) => serialiseLifecycle(s, formatNumberForC))),
		];

		mkdirSync("build/fpu-spike", { recursive: true });
		writeFileSync("build/fpu-spike/generators.json", JSON.stringify(results, null, "\t"));

		for (const r of results) {
			const summary = Object.entries(r.byVariant).map(([name, v]) => `${name}: ${v.differing}/${r.count} ${JSON.stringify(v.kinds)}`);
			console.log(`${r.corpus}\n  ${summary.join("\n  ")}`);
			expect(r.byVariant.canonical.differing).toBe(0);
		}
	});
});
