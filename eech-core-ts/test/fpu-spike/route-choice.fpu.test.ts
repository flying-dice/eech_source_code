//
// Slice 6b route-choice canaries for #9 (x87 intermediate precision): each
// canary of test/scenarios/supply-task-transaction.cases.ts chooses the
// recorded sample under every floating-point variant of the investigation
// oracle. The canonical choice is also checked by the conformance suites; this
// pins that the others still differ, so a change of the numerical contract
// shows up as a changed route.
//

import { describe, expect, it } from "vitest";
import { formatNumberForC } from "../c-reference/c-harness";
import { serialiseLifecycle } from "../scenarios/lifecycle-scenario";
import { ROUTE_CHOICE_CANARIES, routeChoiceCanarySpec, routeChoiceWaypoint } from "../scenarios/supply-task-transaction.cases";
import { runBinary, variantBinary } from "./run";

describe("route-choice canaries under each floating-point model", () => {
	for (const canary of ROUTE_CHOICE_CANARIES) {
		for (const [variant, sample] of Object.entries(canary.choices)) {
			it(`${canary.id}: ${variant} chooses sample ${sample}`, () => {
				const run = runBinary(variantBinary(variant), serialiseLifecycle(routeChoiceCanarySpec(canary.elevations), formatNumberForC));

				expect(run.stdout.split("\n").some((line) => line.startsWith(`${routeChoiceWaypoint(sample)} `))).toBe(true);
			});
		}
	}
});
