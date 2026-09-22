//
// Deterministic EntityReplication: records every transmitted value in order.
//

import type { FloatType } from "../../src/generated/c-enums";
import type { EntityReplication } from "../../src/ports";

export interface FloatValueTransmission {
	entityIndex: number;
	type: FloatType;
	value: number;
}

export class RecordingEntityReplication implements EntityReplication {
	public readonly transmissions: FloatValueTransmission[] = [];

	public transmitEntityFloatValue(entityIndex: number, type: FloatType, value: number): void {
		this.transmissions.push({ entityIndex, type, value });
	}
}
