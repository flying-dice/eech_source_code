import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";


// Runs the corpus exporter with eech-core-ts's toolchain:
//   cd eech-core-ts && npx vitest run --config ../eech-campaign/tools/corpus/vitest.config.mts
const here = dirname(fileURLToPath(import.meta.url));

export default {
	root: join(here, "../../../eech-core-ts"),
	test: {
		include: [join(here, "export.corpus.ts")],
		testTimeout: 1_800_000,
	},
};
