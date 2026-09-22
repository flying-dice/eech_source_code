//
// Mobile entities (aircraft and vehicles) as seen by the campaign.
//
// C provenance: entity/mobile/aircraft/ac_list.c, ac_vec3d.c, ac_funcs.c (overload_aircraft_functions)
//               entity/mobile/vehicle/vh_list.c,  vh_vec3d.c, vh_funcs.c  (overload_vehicle_functions)
//
// Only the campaign-visible surface is ported: membership of a group
// (LIST_TYPE_MEMBER_LINK) and the current position. The position is physical
// state produced by the flight model / vehicle simulation, so it is read
// through the MobilePhysicalState port. Flight models, movement, weapons and
// rendering are excluded (physical simulation).
//

import { toFloat32 } from "../../core/float32";
import { EntityType, ListType, Vec3dType } from "../../generated/c-enums";
import { overloadEntityListLink } from "../system/en_list";
import { fnGetLocalEntityVec3dPtr } from "../system/en_values";
import { getCampaignPorts } from "../system/entity";

// C provenance: hc_funcs.c, fw_funcs.c (overload_aircraft_functions), rv_funcs.c, sh_funcs.c,
//               aa_funcs.c, ps_funcs.c (overload_vehicle_functions)
export const MOBILE_GROUP_MEMBER_ENTITY_TYPES: EntityType[] = [
	EntityType.ENTITY_TYPE_HELICOPTER,
	EntityType.ENTITY_TYPE_FIXED_WING,
	EntityType.ENTITY_TYPE_ROUTED_VEHICLE,
	EntityType.ENTITY_TYPE_SHIP_VEHICLE,
	EntityType.ENTITY_TYPE_ANTI_AIRCRAFT,
	EntityType.ENTITY_TYPE_PERSON,
];

export function overloadMobileFunctions(): void {
	for (const type of MOBILE_GROUP_MEMBER_ENTITY_TYPES) {
		// C provenance: ac_list.c / vh_list.c :: #define LIST_TYPE_MEMBER_LINK
		overloadEntityListLink(type, "member_link", [ListType.LIST_TYPE_MEMBER]);

		// C provenance: ac_vec3d.c / vh_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION) -> &raw->mob.position
		// raw->mob.position is a float vec3d, hence the narrowing.
		fnGetLocalEntityVec3dPtr.overload(type, Vec3dType.VEC3D_TYPE_POSITION, (en) => {
			const position = getCampaignPorts().mobilePhysicalState.getMobilePosition(en.index);

			return {
				x: toFloat32(position.x),
				y: toFloat32(position.y),
				z: toFloat32(position.z),
			};
		});
	}
}
