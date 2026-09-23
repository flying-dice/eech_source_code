import { describe, expect, it } from "vitest";
import { UPDATE_TIMELINE_CASES } from "../scenarios/update-timeline.cases";
import { runTimeline } from "../scenarios/update-timeline";
import { C_REFERENCE_RANDOM_TIMELINES } from "../scenarios/generated/c-reference-random-timelines.cases";

describe("update timeline conformance (JavaScript)", () => {
	for (const c of UPDATE_TIMELINE_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			expect(runTimeline(c.spec)).toEqual(c.expected);
		});
	}
});

describe("random timelines recorded from the original C (JavaScript)", () => {
	it(`all ${C_REFERENCE_RANDOM_TIMELINES.length} match`, () => {
		for (const c of C_REFERENCE_RANDOM_TIMELINES) {
			expect(runTimeline(c.spec), c.id).toEqual(c.expected);
		}
	});
});
