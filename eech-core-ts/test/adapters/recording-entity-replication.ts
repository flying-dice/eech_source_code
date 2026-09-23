//
// Deterministic EntityReplication: records every transmitted value in order.
//

import type { EntityType, FloatType } from "../../src/generated/c-enums";
import type { EntityReplication, ReplicatedEntityAttribute } from "../../src/ports";

export interface FloatValueTransmission {
	entityIndex: number;
	type: FloatType;
	value: number;
}

// Every transmission in order, lifecycle ones included.
export type ReplicationEvent =
	| { kind: "float_value"; entityIndex: number; type: FloatType; value: number }
	| { kind: "create"; type: EntityType; entityIndex: number; attributes: ReplicatedEntityAttribute[] }
	| { kind: "destroy"; entityIndex: number };

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
}
