//
// The observation points of Slice 5a (issue #12), shared by the scenario
// runners. c-reference/harness.c makes the same two observations of the
// original C:
//
// - every FORCE_LOW_ON_SUPPLIES delivery keeps the trace line Slices 1 and 4
//   recorded at that boundary before the force response was ported
//   ("message <receiver> <sender> <message> <sub type>"), and then runs the
//   ported response;
// - taskgen.c :: create_supply_task (Slice 5b) is the new boundary. The
//   port fails loudly there; a test installs the port's interceptor for that
//   one function, which records the call and returns NULL, as the harness's
//   stub does. A scenario that observes it prints "create-supply-task
//   <requester> <supplier> <cargo> <movement> <priority bits> <start keysite>
//   <end keysite>".
//
// TSTL-compatible.
//

import { EntityMessage, EntityType } from "../../src/generated/c-enums";
import { messageResponses } from "../../src/entity/system/en_msgs";
import { interceptCreateSupplyTask } from "../../src/ai/taskgen/taskgen";
import type { Entity, MessageArg } from "../../src/entity/system/entity";
import { float32Hex } from "./float-bits";

export interface LowOnSuppliesDelivery {
	receiver: Entity;
	sender: Entity | undefined;
	message: number;
	subType: number;
}

// Wraps the installed response: onDelivery sees each delivery before the response runs.
export function traceForceLowOnSupplies(onDelivery: (delivery: LowOnSuppliesDelivery) => void): void {
	const FORCE = EntityType.ENTITY_TYPE_FORCE;
	const LOW = EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES;

	const response = messageResponses.lookup(FORCE, LOW, "ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES");

	messageResponses.overload(FORCE, LOW, (message: EntityMessage, receiver: Entity, sender: Entity | undefined, args: MessageArg[]) => {
		onDelivery({ receiver, sender, message, subType: args[0] as number });

		return response(message, receiver, sender, args);
	});
}

//
// Stands in for create_supply_task: each call is passed to onCall as its
// boundary line, and returns NULL (undefined), as the C harness's stub does.
//
export function interceptSupplyTasks(labelOf: (en: Entity | undefined) => string, onCall: (line: string) => void): void {
	interceptCreateSupplyTask((requester, supplier, cargo, movement_type, priority, start_keysite, end_keysite) => {
		onCall(
			`create-supply-task ${labelOf(requester)} ${labelOf(supplier)} ${labelOf(cargo)} ${movement_type} ${float32Hex(priority)} ` +
				`${labelOf(start_keysite)} ${labelOf(end_keysite)}`,
		);

		return undefined;
	});
}
