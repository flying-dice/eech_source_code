//
// Port: physical state of mobile entities (aircraft and vehicles).
//
// C provenance: entity/mobile/aircraft/ac_vec3d.c :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)
//               entity/mobile/vehicle/vh_vec3d.c  :: get_local_vec3d_ptr (VEC3D_TYPE_POSITION)
//
// In EECH `raw->mob.position` is written by the flight model / vehicle
// movement simulation. The campaign only reads it. The physical simulation is
// supplied by the environment (DCS in production, a deterministic adapter in
// tests), so the campaign core asks this port instead.
//
// Values are narrowed to C `float` by the core, not by the adapter.
//

import type { Vec3d } from "../core/maths/vec3d";

export interface MobilePhysicalState {
	// Current world position of the mobile entity with the given EECH entity index.
	getMobilePosition(entityIndex: number): Vec3d;
}
