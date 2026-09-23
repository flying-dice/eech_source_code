//
// The observation points of Slice 5a (issue #12), shared by the scenario
// runners. c-reference/harness.c makes the same two observations of the
// original C:
//
// - every FORCE_LOW_ON_SUPPLIES delivery keeps the trace line Slices 1 and 4
//   recorded at that boundary before the force response was ported
//   ("message <receiver> <sender> <message> <sub type>"), and then runs the
//   ported response;
// - taskgen.c :: create_supply_task (Slice 5b) is the new boundary. The port
//   records the call (callUnportedFunction); a scenario that observes it
//   prints "create-supply-task <requester> <supplier> <cargo> <movement>
//   <priority bits> <start keysite> <end keysite>".
//
// TSTL-compatible.
//

import { EntityMessage, EntityType } from "../../src/generated/c-enums";
import { messageResponses } from "../../src/entity/system/en_msgs";
import { takeUnportedCallLog, type Entity, type MessageArg } from "../../src/entity/system/entity";
import { float32Hex } from "./float-bits";

export interface LowOnSuppliesDelivery {
	receiver: Entity;
	sender: Entity | undefined;
	message: number;
	subType: number;
}

//
// Wraps the installed response: onDelivery sees the delivery before the
// response runs, onReturn after it (with the boundary calls it made).
//
export function traceForceLowOnSupplies(onDelivery: (delivery: LowOnSuppliesDelivery) => void, onReturn: () => void): void {
	const FORCE = EntityType.ENTITY_TYPE_FORCE;
	const LOW = EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES;

	const response = messageResponses.lookup(FORCE, LOW, "ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES");

	messageResponses.overload(FORCE, LOW, (message: EntityMessage, receiver: Entity, sender: Entity | undefined, args: MessageArg[]) => {
		onDelivery({ receiver, sender, message, subType: args[0] as number });

		const result = response(message, receiver, sender, args);

		onReturn();

		return result;
	});
}

// Boundary lines for the create_supply_task calls recorded since the last take.
export function takeSupplyTaskLines(labelOf: (en: Entity | undefined) => string): string[] {
	const lines: string[] = [];

	for (const call of takeUnportedCallLog()) {
		if (call.name !== "create_supply_task") {
			throw new Error(`unexpected unported call ${call.name}`);
		}

		const a = call.args;

		lines.push(
			`create-supply-task ${labelOf(a[0] as Entity)} ${labelOf(a[1] as Entity)} ${labelOf(a[2] as Entity)} ${a[3] as number} ${float32Hex(a[4] as number)} ` +
				`${labelOf(a[5] as Entity | undefined)} ${labelOf(a[6] as Entity | undefined)}`,
		);
	}

	return lines;
}
