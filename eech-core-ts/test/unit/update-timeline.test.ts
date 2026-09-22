import { describe, expect, it } from "vitest";
import { UPDATE_TIMELINE_CASES } from "../scenarios/update-timeline.cases";
import { runTimeline } from "../scenarios/update-timeline";

describe("update timeline conformance (JavaScript)", () => {
	for (const c of UPDATE_TIMELINE_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			expect(runTimeline(c.spec)).toEqual(c.expected);
		});
	}
});
