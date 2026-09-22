//
// Typed entity values.
//
// C provenance: entity/system/en_funcs/en_int.c, en_float.c, en_vec3d.c, en_ptr.c
//               and the dispatch macros in the matching headers, e.g.
//   #define get_local_entity_int_value(EN,INT_TYPE) \
//     (fn_get_local_entity_int_value[get_local_entity_type ((EN))][validate_int_type ((INT_TYPE))] ((EN), (INT_TYPE)))
//   #define set_client_server_entity_float_value(EN,FLOAT_TYPE,VALUE) \
//     (fn_set_client_server_entity_float_value[get_local_entity_type ((EN))][...][get_comms_model ()] (...))
//

import { toFloat32 } from "../../core/float32";
import type { Vec3d } from "../../core/maths/vec3d";
import { CommsModelType, FloatType, IntType, PtrType, Vec3dType } from "../../generated/c-enums";
import { getCommsModel, type CommsModel } from "./comms";
import { getCampaignPorts, type Entity } from "./entity";
import { EntityFunctionTable } from "./function-table";

export type GetIntValueFn = (en: Entity, type: IntType) => number;
export type GetFloatValueFn = (en: Entity, type: FloatType) => number;
export type SetFloatValueFn = (en: Entity, type: FloatType, value: number) => void;
export type GetVec3dPtrFn = (en: Entity, type: Vec3dType) => Vec3d | undefined;
export type GetPtrValueFn = (en: Entity, type: PtrType) => Entity | undefined;

export const fnGetLocalEntityIntValue = new EntityFunctionTable<GetIntValueFn>("fn_get_local_entity_int_value");

export const fnGetLocalEntityFloatValue = new EntityFunctionTable<GetFloatValueFn>("fn_get_local_entity_float_value");

export const fnSetClientServerEntityFloatValue: Record<CommsModel, EntityFunctionTable<SetFloatValueFn>> = {
	[CommsModelType.COMMS_MODEL_SERVER]: new EntityFunctionTable<SetFloatValueFn>("fn_set_client_server_entity_float_value [COMMS_MODEL_SERVER]"),
	[CommsModelType.COMMS_MODEL_CLIENT]: new EntityFunctionTable<SetFloatValueFn>("fn_set_client_server_entity_float_value [COMMS_MODEL_CLIENT]"),
};

export const fnGetLocalEntityVec3dPtr = new EntityFunctionTable<GetVec3dPtrFn>("fn_get_local_entity_vec3d_ptr");

export const fnGetLocalEntityPtrValue = new EntityFunctionTable<GetPtrValueFn>("fn_get_local_entity_ptr_value");

export function getLocalEntityIntValue(en: Entity, type: IntType): number {
	return fnGetLocalEntityIntValue.lookup(en.type, type, IntType[type])(en, type);
}

export function getLocalEntityFloatValue(en: Entity, type: FloatType): number {
	return fnGetLocalEntityFloatValue.lookup(en.type, type, FloatType[type])(en, type);
}

// The C prototype takes `float value`, so the argument is narrowed on entry.
export function setClientServerEntityFloatValue(en: Entity, type: FloatType, value: number): void {
	fnSetClientServerEntityFloatValue[getCommsModel()].lookup(en.type, type, FloatType[type])(en, type, toFloat32(value));
}

export function getLocalEntityVec3dPtr(en: Entity, type: Vec3dType): Vec3d | undefined {
	return fnGetLocalEntityVec3dPtr.lookup(en.type, type, Vec3dType[type])(en, type);
}

export function getLocalEntityPtrValue(en: Entity, type: PtrType): Entity | undefined {
	return fnGetLocalEntityPtrValue.lookup(en.type, type, PtrType[type])(en, type);
}

//
// C provenance: the set_server_float_value found in every xx_float.c:
//
//   static void set_server_float_value (entity *en, float_types type, float value)
//   {
//     validate_client_server_local_fn ();
//     set_local_float_value (en, type, value);
//     validate_client_server_remote_fn ();
//     set_remote_float_value (en, type, value);   // transmit_entity_comms_message (ENTITY_COMMS_FLOAT_VALUE, ...)
//   }
//
// validate_client_server_*_fn are debug-build checks of the comms dispatch
// context and are not ported.
//
export function serverFloatValueSetter(setLocalFloatValue: SetFloatValueFn): SetFloatValueFn {
	return (en, type, value) => {
		setLocalFloatValue(en, type, value);

		getCampaignPorts().entityReplication.transmitEntityFloatValue(en.index, type, value);
	};
}
