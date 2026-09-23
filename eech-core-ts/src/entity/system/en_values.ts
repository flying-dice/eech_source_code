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

import { toFloat32RTZ } from "../../core/float32";
import type { Vec3d } from "../../core/maths/vec3d";
import { CommsModelType, FloatType, IntType, PtrType, Vec3dType } from "../../generated/c-enums";
import { getCommsModel, type CommsModel } from "./comms";
import { transmitEntityFloatValue } from "./en_comms";
import type { Entity } from "./entity";
import { EntityFunctionTable } from "./function-table";

export type GetIntValueFn = (en: Entity, type: IntType) => number;
export type SetIntValueFn = (en: Entity, type: IntType, value: number) => void;
export type GetFloatValueFn = (en: Entity, type: FloatType) => number;
export type SetFloatValueFn = (en: Entity, type: FloatType, value: number) => void;
export type GetVec3dPtrFn = (en: Entity, type: Vec3dType) => Vec3d | undefined;
export type SetVec3dFn = (en: Entity, type: Vec3dType, v: Vec3d) => void;
export type GetPtrValueFn = (en: Entity, type: PtrType) => Entity | undefined;
// C: void (*) (entity *en, ptr_types type, void *ptr)
export type SetPtrValueFn = (en: Entity, type: PtrType, ptr: unknown) => void;

export const fnGetLocalEntityIntValue = new EntityFunctionTable<GetIntValueFn>("fn_get_local_entity_int_value");

export const fnSetLocalEntityIntValue = new EntityFunctionTable<SetIntValueFn>("fn_set_local_entity_int_value");

export const fnSetLocalEntityRawIntValue = new EntityFunctionTable<SetIntValueFn>("fn_set_local_entity_raw_int_value");

export const fnSetLocalEntityRawFloatValue = new EntityFunctionTable<SetFloatValueFn>("fn_set_local_entity_raw_float_value");

export const fnSetLocalEntityRawVec3d = new EntityFunctionTable<SetVec3dFn>("fn_set_local_entity_raw_vec3d");

export const fnGetLocalEntityFloatValue = new EntityFunctionTable<GetFloatValueFn>("fn_get_local_entity_float_value");

export const fnSetLocalEntityFloatValue = new EntityFunctionTable<SetFloatValueFn>("fn_set_local_entity_float_value");

export const fnSetLocalEntityPtrValue = new EntityFunctionTable<SetPtrValueFn>("fn_set_local_entity_ptr_value");

export const fnSetClientServerEntityFloatValue: Record<CommsModel, EntityFunctionTable<SetFloatValueFn>> = {
	[CommsModelType.COMMS_MODEL_SERVER]: new EntityFunctionTable<SetFloatValueFn>("fn_set_client_server_entity_float_value [COMMS_MODEL_SERVER]"),
	[CommsModelType.COMMS_MODEL_CLIENT]: new EntityFunctionTable<SetFloatValueFn>("fn_set_client_server_entity_float_value [COMMS_MODEL_CLIENT]"),
};

export const fnGetLocalEntityVec3dPtr = new EntityFunctionTable<GetVec3dPtrFn>("fn_get_local_entity_vec3d_ptr");

export const fnGetLocalEntityPtrValue = new EntityFunctionTable<GetPtrValueFn>("fn_get_local_entity_ptr_value");

export function getLocalEntityIntValue(en: Entity, type: IntType): number {
	return fnGetLocalEntityIntValue.lookup(en.type, type, IntType[type])(en, type);
}

// The C prototype takes `int value`.
export function setLocalEntityIntValue(en: Entity, type: IntType, value: number): void {
	fnSetLocalEntityIntValue.lookup(en.type, type, IntType[type])(en, type, value);
}

// The C prototype takes `int value`.
export function setLocalEntityRawIntValue(en: Entity, type: IntType, value: number): void {
	fnSetLocalEntityRawIntValue.lookup(en.type, type, IntType[type])(en, type, value);
}

// The C prototype takes `float value`, so the argument is narrowed on entry
// (toward zero: EECH's FPU rounding, docs/fidelity/fpu-semantics.md).
export function setLocalEntityRawFloatValue(en: Entity, type: FloatType, value: number): void {
	fnSetLocalEntityRawFloatValue.lookup(en.type, type, FloatType[type])(en, type, toFloat32RTZ(value));
}

// The C prototype takes `vec3d *v`; vec3d members are floats.
export function setLocalEntityRawVec3d(en: Entity, type: Vec3dType, v: Vec3d): void {
	fnSetLocalEntityRawVec3d.lookup(en.type, type, Vec3dType[type])(en, type, v);
}

// C provenance: en_int.c :: default_get_entity_int_value, `default:` arm
// (value = 0). Only installed for int types whose C default is that arm, on
// entity types whose overload tables are known to keep the default.
export function defaultGetEntityIntValue(_en: Entity, _type: IntType): number {
	return 0;
}

// C provenance: en_float.c :: default_get_entity_float_value, `default:` arm
// (value = 0.0). Only installed for float types whose C default is that arm, on
// entity types whose overload tables are known to keep the default.
export function defaultGetEntityFloatValue(_en: Entity, _type: FloatType): number {
	return 0.0;
}

// C provenance: en_int.c :: default_set_entity_int_value (does nothing). Only
// installed where the C overload tables are known to keep this default.
export function defaultSetEntityIntValue(_en: Entity, _type: IntType, _value: number): void {}

// The C prototype takes `float value`, so the argument is narrowed on entry (toward zero).
export function setLocalEntityFloatValue(en: Entity, type: FloatType, value: number): void {
	fnSetLocalEntityFloatValue.lookup(en.type, type, FloatType[type])(en, type, toFloat32RTZ(value));
}

// The C prototype takes `void *ptr`.
export function setLocalEntityPtrValue(en: Entity, type: PtrType, ptr: unknown): void {
	fnSetLocalEntityPtrValue.lookup(en.type, type, PtrType[type])(en, type, ptr);
}

export function getLocalEntityFloatValue(en: Entity, type: FloatType): number {
	return fnGetLocalEntityFloatValue.lookup(en.type, type, FloatType[type])(en, type);
}

// The C prototype takes `float value`, so the argument is narrowed on entry (toward zero).
export function setClientServerEntityFloatValue(en: Entity, type: FloatType, value: number): void {
	fnSetClientServerEntityFloatValue[getCommsModel()].lookup(en.type, type, FloatType[type])(en, type, toFloat32RTZ(value));
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

		transmitEntityFloatValue(en, type, value);
	};
}
