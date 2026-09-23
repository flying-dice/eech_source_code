// INVESTIGATION ONLY (issue #14, F1): builds the canonical harness and the probe variants once.

// @ts-expect-error plain ESM build script without type declarations
import { buildHarness, buildHarnessF1Variant } from "../../c-reference/build.mjs";
import { F1_VARIANTS } from "./variants";

export default function setup(): void {
	(buildHarness as () => string)();

	for (const variant of F1_VARIANTS) {
		(buildHarnessF1Variant as (v: unknown) => string)(variant);
	}
}
