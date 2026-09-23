//
// Port: outbound replication of authoritative entity changes.
//
// C provenance: entity/system/en_comms/en_comms.c :: transmit_entity_comms_message
//               (called by every set_server_*_value, create_remote and
//               destroy_remote in the entity overloads)
//
// In EECH the server applies a change locally and then transmits it to
// clients (ENTITY_COMMS_FLOAT_VALUE, ENTITY_COMMS_CREATE, ENTITY_COMMS_DESTROY,
// ...). The network transport is not part of the campaign core; the semantic
// contract "this authoritative change happened" is. An adapter may forward it
// to a transport, mirror it into DCS, or drop it.
//

import type { EntityType, FloatType, IntType, ListType, Vec3dType } from "../generated/c-enums";

// One creation attribute as ENTITY_COMMS_CREATE packs it (en_attrs.c ::
// pack_entity_attributes): floats narrowed, entities by index (-1: NULL).
export type ReplicatedEntityAttribute =
	| { kind: "int_value"; type: IntType; value: number }
	| { kind: "float_value"; type: FloatType; value: number }
	| { kind: "vec3d"; type: Vec3dType; x: number; y: number; z: number }
	| { kind: "parent"; type: ListType; entityIndex: number }
	| { kind: "child_pred"; type: ListType; entityIndex: number };

export interface EntityReplication {
	// C: transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value)
	transmitEntityFloatValue(entityIndex: number, type: FloatType, value: number): void;

	// C: transmit_entity_comms_message (ENTITY_COMMS_CREATE, NULL, type, index, pargs)
	transmitEntityCreate(type: EntityType, entityIndex: number, attributes: ReplicatedEntityAttribute[]): void;

	// C: transmit_entity_comms_message (ENTITY_COMMS_DESTROY, en)
	transmitEntityDestroy(entityIndex: number): void;
}
