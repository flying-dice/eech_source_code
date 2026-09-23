//
// Entity destruction.
//
// C provenance: entity/system/en_funcs/en_dstry.c, en_dstry.h
//
//   #define destroy_client_server_entity(EN) \
//     (fn_destroy_client_server_entity[get_local_entity_type ((EN))][get_comms_model ()] ((EN)))
//   #define destroy_client_server_entity_family(EN) \
//     (fn_destroy_client_server_entity_family[get_local_entity_type ((EN))][get_comms_model ()] ((EN)))
//
// EECH's defaults (default_destroy_entity, default_destroy_entity_family) do
// nothing. They are installed only for ENTITY_TYPE_UNKNOWN, which nothing
// overloads: destroying an entity that has already been freed does nothing.
// Other entity types without a ported overload fail loudly.
//
// Not ported: update_destroy_entity_statistics (debug display counters), kill,
// destroy_local_entities, destroy_local_only_entities.
//

import { CommsModelType, EntityType } from "../../generated/c-enums";
import { getCommsModel } from "./comms";
import type { Entity } from "./entity";
import { EntityFunctionTable } from "./function-table";

export type DestroyEntityFn = (en: Entity) => void;

// keyed by [entity type][0]
export const fnDestroyLocalEntity = new EntityFunctionTable<DestroyEntityFn>("fn_destroy_local_entity");

// keyed by [entity type][comms model]
export const fnDestroyClientServerEntity = new EntityFunctionTable<DestroyEntityFn>("fn_destroy_client_server_entity");

export const fnDestroyClientServerEntityFamily = new EntityFunctionTable<DestroyEntityFn>("fn_destroy_client_server_entity_family");

// C provenance: en_dstry.c :: default_destroy_entity, default_destroy_entity_family
export function defaultDestroyEntity(_en: Entity): void {}

// C provenance: en_dstry.c :: destroy_local_entity
export function destroyLocalEntity(en: Entity): void {
	fnDestroyLocalEntity.lookup(en.type, 0, "local")(en);
}

export function destroyClientServerEntity(en: Entity): void {
	const model = getCommsModel();

	fnDestroyClientServerEntity.lookup(en.type, model, CommsModelType[model])(en);
}

export function destroyClientServerEntityFamily(en: Entity): void {
	const model = getCommsModel();

	fnDestroyClientServerEntityFamily.lookup(en.type, model, CommsModelType[model])(en);
}

// C provenance: en_dstry.c :: initialise_entity_destroy_default_functions, for
// the entity type no overload replaces
export function overloadUnknownEntityDestroyFunctions(): void {
	const UNKNOWN = EntityType.ENTITY_TYPE_UNKNOWN;

	fnDestroyLocalEntity.overload(UNKNOWN, 0, defaultDestroyEntity);

	for (const model of [CommsModelType.COMMS_MODEL_SERVER, CommsModelType.COMMS_MODEL_CLIENT]) {
		fnDestroyClientServerEntity.overload(UNKNOWN, model, defaultDestroyEntity);
		fnDestroyClientServerEntityFamily.overload(UNKNOWN, model, defaultDestroyEntity);
	}
}
