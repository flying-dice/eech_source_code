//
// Entity creation.
//
// C provenance: entity/system/en_funcs/en_creat.c, en_creat.h;
//               entity/system/en_debug/en_valid.c (create index checks)
//
// create_local_entity / create_client_server_entity dispatch through
// fn_create_local_entity[type] and fn_create_client_server_entity[type][comms_model]
// with the attribute list (en_attrs.ts). EECH's default, default_create_entity,
// is debug_fatal; the port leaves unported entries unported instead.
//
// Not ported: update_create_entity_statistics (debug display counters with no
// campaign effect; its ASSERT repeats the entity type check below), the
// variadic / received-buffer conversion (the port takes the list directly),
// force_local_entity_create_stack_attributes (never set on the server's TX
// path), and create_local_only_entities (pylons, bridges, camera).
//

import { ASSERT, EechFatalError, UnportedBehaviourError } from "../../core/assert";
import { CommsModelType, EntityType } from "../../generated/c-enums";
import { getCommsModel, type CommsModel } from "./comms";
import type { EntityAttribute } from "./en_attrs";
import { ENTITY_INDEX_DONT_CARE } from "./en_heap";
import type { Entity } from "./entity";
import { EntityFunctionTable } from "./function-table";

export type CreateEntityFn = (type: EntityType, index: number, attributes: EntityAttribute[]) => Entity | undefined;

// keyed by [entity type][0]
export const fnCreateLocalEntity = new EntityFunctionTable<CreateEntityFn>("fn_create_local_entity");

// keyed by [entity type][comms model]
export const fnCreateClientServerEntity = new EntityFunctionTable<CreateEntityFn>("fn_create_client_server_entity");

function assertEntityType(type: EntityType): void {
	ASSERT(type > EntityType.ENTITY_TYPE_UNKNOWN && type < EntityType.NUM_ENTITY_TYPES, "(type > ENTITY_TYPE_UNKNOWN) && (type < NUM_ENTITY_TYPES)");
}

// C provenance: en_creat.c :: create_local_entity
export function createLocalEntity(type: EntityType, index: number, attributes: EntityAttribute[]): Entity {
	assertEntityType(type);

	const en = fnCreateLocalEntity.lookup(type, 0, "local")(type, index, attributes);

	if (!en) {
		throw new EechFatalError("EN_CREATE: CREATE_LOCAL_ENTITY : unable to create entity %s. Limit of %d reached");
	}

	return en;
}

// C provenance: en_creat.c :: create_client_server_entity
export function createClientServerEntity(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	assertEntityType(type);

	const model: CommsModel = getCommsModel();

	const en = fnCreateClientServerEntity.lookup(type, model, CommsModelType[model])(type, index, attributes);

	if (model === CommsModelType.COMMS_MODEL_SERVER && !en) {
		throw new EechFatalError("EN_CREATE: CREATE_CLIENT_SERVER_ENTITY : unable to create entity %s. Limit of %d reached");
	}

	return en;
}

// C provenance: en_valid.h :: validate_local_create_entity_index, en_valid.c ::
// assert_local_create_entity_index (debug-build ASSERT), downwash off. The
// server needs ENTITY_INDEX_DONT_CARE. A client never accepts it (TX: FALSE;
// RX: index != ENTITY_INDEX_DONT_CARE); a client's given index depends on the
// comms data flow, which is not ported.
export function validateLocalCreateEntityIndex(index: number): void {
	if (getCommsModel() === CommsModelType.COMMS_MODEL_SERVER) {
		ASSERT(index === ENTITY_INDEX_DONT_CARE, "assert_local_create_entity_index ((index))");
	} else {
		ASSERT(index !== ENTITY_INDEX_DONT_CARE, "assert_local_create_entity_index ((index))");

		throw new UnportedBehaviourError("en_valid.c :: assert_local_create_entity_index: a client's given index (get_comms_data_flow)");
	}
}

// C provenance: en_valid.h :: validate_remote_create_entity_index, en_valid.c ::
// assert_remote_create_entity_index (server case)
export function validateRemoteCreateEntityIndex(index: number): void {
	ASSERT(index !== ENTITY_INDEX_DONT_CARE, "assert_remote_create_entity_index ((index))");
}
