import { describe, expect, it } from "vitest";
import { UPDATE_TIMELINE_CASES } from "../scenarios/update-timeline.cases";
import { runCTimeline } from "./c-harness";

// Every expected timeline outcome must be what the original C does.
describe("update timeline expectations hold for the original EECH C", () => {
	for (const c of UPDATE_TIMELINE_CASES) {
		it(c.id, () => {
			expect(runCTimeline(c.spec)).toEqual(c.expected);
		});
	}
});
