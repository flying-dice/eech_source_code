import { expect, it } from "vitest";
import { runTimeline } from "../scenarios/update-timeline";
import { runCTimeline } from "./c-harness";
import { generateRandomTimelines } from "./random-scenarios";

it("probe: TS timelines match the original C", () => {
	let mismatches = 0;
	for (const spec of generateRandomTimelines(7, 400)) {
		const c = runCTimeline(spec);
		const ts = runTimeline(spec);
		try {
			expect(ts).toEqual(c);
		} catch (e) {
			if (mismatches++ < 2) {
				console.log(JSON.stringify(spec), "\nC:", JSON.stringify(c), "\nTS:", JSON.stringify(ts));
			}
		}
	}
	expect(mismatches).toBe(0);
});
