//
// INVESTIGATION ONLY (issue #7): builds the canonical harness and every
// floating-point variant once, before the spike runs.
//

// @ts-expect-error plain ESM build script without type declarations
import { buildHarness, buildHarnessVariant } from "../../c-reference/build.mjs";
// @ts-expect-error plain ESM module without type declarations
import { FPU_VARIANTS } from "../../c-reference/fpu-variants.mjs";

export default function setup(): void {
	buildHarness();
	for (const variant of FPU_VARIANTS as { name: string }[]) {
		buildHarnessVariant(variant);
	}
}
