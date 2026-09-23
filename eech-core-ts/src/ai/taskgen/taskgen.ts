//
// Task generation.
//
// C provenance: ai/taskgen/taskgen.c
//
// Only the boundary Slice 5a (issue #12) stops at is here.
//

import { UnportedBehaviourError } from "../../core/assert";
import type { MovementType } from "../../generated/c-enums";
import type { Entity } from "../../entity/system/entity";

// The arguments of taskgen.c :: create_supply_task, in C order.
export type CreateSupplyTaskFn = (
	requester: Entity,
	supplier: Entity,
	cargo: Entity,
	movement_type: MovementType,
	priority: number,
	start_keysite: Entity | undefined,
	end_keysite: Entity | undefined,
) => Entity | undefined;

let interceptor: CreateSupplyTaskFn | undefined;

//
// Test seam for this one boundary. A conformance test installs a stand-in
// that records the call and returns what the test decides (the C harness's
// stub returns NULL). It is not part of the public API, it cannot make any
// other unported function return, and initialiseCampaignCore removes it.
//
export function interceptCreateSupplyTask(fn: CreateSupplyTaskFn): void {
	interceptor = fn;
}

export function resetCreateSupplyTaskInterceptor(): void {
	interceptor = undefined;
}

//
// C provenance: taskgen.c :: create_supply_task
//
// Not ported: task construction (create_supply_task -> create_task -> the
// task entity, start-keysite scoring, difficulty and route construction) is
// Slice 5b (issue #14). Reaching it fails loudly, as all unported campaign
// behaviour does; only an installed test interceptor answers instead.
//
export function createSupplyTask(
	requester: Entity,
	supplier: Entity,
	cargo: Entity,
	movement_type: MovementType,
	priority: number,
	start_keysite: Entity | undefined,
	end_keysite: Entity | undefined,
): Entity | undefined {
	if (interceptor === undefined) {
		throw new UnportedBehaviourError("taskgen.c :: create_supply_task (Slice 5b)");
	}

	return interceptor(requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite);
}
