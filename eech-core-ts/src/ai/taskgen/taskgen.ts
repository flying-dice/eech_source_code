//
// Task generation.
//
// C provenance: ai/taskgen/taskgen.c
//
// Only the boundary Slice 5a (issue #12) stops at is here.
//

import type { MovementType } from "../../generated/c-enums";
import { callUnportedFunction, type Entity } from "../../entity/system/entity";

//
// C provenance: taskgen.c :: create_supply_task
//
// Not ported: task construction (create_supply_task -> create_task -> the
// task entity, start-keysite scoring, difficulty and route construction) is
// Slice 5b. The call is the boundary of Slice 5a: it fails loudly in
// production and is recorded with its arguments in tests. It returns NULL;
// response_to_force_low_on_supplies, its only caller, reads the result only
// in DEBUG_SUPPLY logging, which is compiled out.
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
	callUnportedFunction({
		name: "create_supply_task",
		args: [requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite],
		provenance: "taskgen.c :: create_supply_task (Slice 5b)",
	});

	return undefined;
}
