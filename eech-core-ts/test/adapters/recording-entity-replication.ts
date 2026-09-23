//
// Deterministic EntityReplication: records every transmitted value in order.
//

import type { EntityType, FloatType, ListType } from "../../src/generated/c-enums";
import type { EntityReplication, ReplicatedEntityAttribute, ReplicatedTaskRoute } from "../../src/ports";

export interface FloatValueTransmission {
	entityIndex: number;
	type: FloatType;
	value: number;
}

// Every transmission in order, lifecycle ones included.
export type ReplicationEvent =
	| { kind: "float_value"; entityIndex: number; type: FloatType; value: number }
	| { kind: "create"; type: EntityType; entityIndex: number; attributes: ReplicatedEntityAttribute[] }
	| { kind: "destroy"; entityIndex: number }
	| { kind: "task_pointers"; taskIndex: number; route: ReplicatedTaskRoute }
	| { kind: "switch_parent"; entityIndex: number; type: ListType; parentIndex: number };

export class RecordingEntityReplication implements EntityReplication {
	public readonly transmissions: FloatValueTransmission[] = [];

	public readonly events: ReplicationEvent[] = [];

	public transmitEntityFloatValue(entityIndex: number, type: FloatType, value: number): void {
		this.transmissions.push({ entityIndex, type, value });
		this.events.push({ kind: "float_value", entityIndex, type, value });
	}

	public transmitEntityCreate(type: EntityType, entityIndex: number, attributes: ReplicatedEntityAttribute[]): void {
		this.events.push({ kind: "create", type, entityIndex, attributes });
	}

	public transmitEntityDestroy(entityIndex: number): void {
		this.events.push({ kind: "destroy", entityIndex });
	}

	public transmitTaskPointers(taskIndex: number, route: ReplicatedTaskRoute): void {
		this.events.push({ kind: "task_pointers", taskIndex, route });
	}

	public transmitSwitchParent(entityIndex: number, type: ListType, parentIndex: number): void {
		this.events.push({ kind: "switch_parent", entityIndex, type, parentIndex });
	}
}
