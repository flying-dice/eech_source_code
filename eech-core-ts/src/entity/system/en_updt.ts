//
// Entity update dispatch.
//
// C provenance: entity/system/en_funcs/en_updt.c, en_updt.h
//
//   #define update_client_server_entity(EN) \
//     (fn_update_client_server_entity[get_local_entity_type ((EN))][get_comms_model ()] ((EN)))
//
// EECH's default (default_update_entity) does nothing. It is not installed:
// an entity type whose update overload is not ported fails loudly.
//

import { CommsModelType } from "../../generated/c-enums";
import { getCommsModel } from "./comms";
import type { Entity } from "./entity";
import { EntityFunctionTable } from "./function-table";

export type UpdateFn = (en: Entity) => void;

// keyed by [entity type][comms model]
export const fnUpdateClientServerEntity = new EntityFunctionTable<UpdateFn>("fn_update_client_server_entity");

export function updateClientServerEntity(en: Entity): void {
	const model = getCommsModel();

	fnUpdateClientServerEntity.lookup(en.type, model, CommsModelType[model])(en);
}
