//
// Entity creation attributes.
//
// C provenance: entity/system/en_attrs/en_attrs.c :: set_local_entity_attributes,
//               en_attrs.h (ENTITY_ATTR_* macros)
//
// EECH passes attributes as a variadic list (or, on receive, a buffer of the
// same layout) terminated by ENTITY_ATTR_END, and reads it with get_list_item.
// Floats arrive promoted to double. The port takes the list as an array of
// tagged attributes, in order, without the terminator.
//
// Each attribute calls the entity type's *raw* setter, or sets a list link's
// parent / predecessor pointer without inserting the entity into the list.
//
// Attribute kinds without a ported setter on any entity type
// (ENTITY_ATTR_ATTITUDE_ANGLES, CHAR_VALUE, PTR_VALUE, STRING) are not
// representable yet.
//

import { toFloat32 } from "../../core/float32";
import type { FloatType, IntType, ListType, Vec3dType } from "../../generated/c-enums";
import type { ReplicatedEntityAttribute } from "../../ports";
import { setLocalEntityChildPred, setLocalEntityParent } from "./en_list";
import { setLocalEntityRawFloatValue, setLocalEntityRawIntValue, setLocalEntityRawVec3d } from "./en_values";
import type { Entity } from "./entity";

export type EntityAttribute =
	// ENTITY_ATTR_INT_VALUE (INT_TYPE, VALUE)
	| { kind: "int_value"; type: IntType; value: number }
	// ENTITY_ATTR_FLOAT_VALUE (FLOAT_TYPE, VALUE): VALUE travels as a double
	| { kind: "float_value"; type: FloatType; value: number }
	// ENTITY_ATTR_VEC3D (VEC3D_TYPE, X, Y, Z): components travel as doubles
	| { kind: "vec3d"; type: Vec3dType; x: number; y: number; z: number }
	// ENTITY_ATTR_PARENT (LIST_TYPE, PARENT)
	| { kind: "parent"; type: ListType; entity: Entity | undefined }
	// ENTITY_ATTR_CHILD_PRED (LIST_TYPE, PRED)
	| { kind: "child_pred"; type: ListType; entity: Entity | undefined };

// C provenance: en_attrs.c :: set_local_entity_attributes
export function setLocalEntityAttributes(en: Entity, attributes: EntityAttribute[]): void {
	for (const attr of attributes) {
		switch (attr.kind) {
			case "int_value": {
				setLocalEntityRawIntValue(en, attr.type, attr.value);

				break;
			}
			case "float_value": {
				// float value = get_list_item (pargs, double)
				setLocalEntityRawFloatValue(en, attr.type, toFloat32(attr.value));

				break;
			}
			case "vec3d": {
				// vec3d v; v.x = get_list_item (pargs, double); ...
				setLocalEntityRawVec3d(en, attr.type, { x: toFloat32(attr.x), y: toFloat32(attr.y), z: toFloat32(attr.z) });

				break;
			}
			case "parent": {
				setLocalEntityParent(en, attr.type, attr.entity);

				break;
			}
			case "child_pred": {
				setLocalEntityChildPred(en, attr.type, attr.entity);

				break;
			}
		}
	}
}

// C provenance: en_attrs.c :: pack_entity_attributes (what ENTITY_COMMS_CREATE carries)
export function replicatedEntityAttributes(attributes: EntityAttribute[]): ReplicatedEntityAttribute[] {
	const replicated: ReplicatedEntityAttribute[] = [];

	for (const attr of attributes) {
		switch (attr.kind) {
			case "int_value": {
				replicated.push({ kind: "int_value", type: attr.type, value: attr.value });

				break;
			}
			case "float_value": {
				replicated.push({ kind: "float_value", type: attr.type, value: toFloat32(attr.value) });

				break;
			}
			case "vec3d": {
				replicated.push({ kind: "vec3d", type: attr.type, x: toFloat32(attr.x), y: toFloat32(attr.y), z: toFloat32(attr.z) });

				break;
			}
			case "parent": {
				replicated.push({ kind: "parent", type: attr.type, entityIndex: attr.entity === undefined ? -1 : attr.entity.index });

				break;
			}
			case "child_pred": {
				replicated.push({ kind: "child_pred", type: attr.type, entityIndex: attr.entity === undefined ? -1 : attr.entity.index });

				break;
			}
		}
	}

	return replicated;
}
