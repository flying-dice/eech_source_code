//
// Builds the C reference harness exactly once, before any test worker starts.
// Test files run in parallel workers; building per file let one worker
// execute the binary while another was rewriting it (EACCES / ENOENT).
//

// @ts-expect-error plain ESM build script without type declarations
import { buildHarness } from "../../c-reference/build.mjs";

export default function setup(): void {
	buildHarness();
}
