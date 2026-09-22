//
// Port: outbound replication of authoritative entity value changes.
//
// C provenance: entity/system/en_comms/en_comms.c :: transmit_entity_comms_message
//               (called by every set_server_*_value in the entity overloads)
//
// In EECH the server applies a value locally and then transmits it to clients
// (ENTITY_COMMS_FLOAT_VALUE, ...). The network transport is not part of the
// campaign core; the semantic contract "this authoritative value changed" is.
// An adapter may forward it to a transport, mirror it into DCS, or drop it.
//

import type { FloatType } from "../generated/c-enums";

export interface EntityReplication {
	// C: transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, en, type, value)
	transmitEntityFloatValue(entityIndex: number, type: FloatType, value: number): void;
}
