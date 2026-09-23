//
// The world map: sector grid and map extents.
//
// C provenance: entity/system/en_main/en_world.c :: set_entity_world_map_size,
//               en_world.h (MIN/MID/MAX_MAP_*, get_x_sector, get_z_sector,
//               point_inside_map_area), misc/miscell.c :: int_bit_count
//
// EECH's campaign script parser sets the map from the terrain's dimensions
// (parsgen.c, parser.c, comm_man.c). These are campaign data the host passes
// in, not measured environment, so they are set through the ported function
// rather than a port.
//
// The extents are C floats. get_x_sector converts with convert_float_to_int,
// which is x87 fistp under EECH's round-toward-zero FPU mode
// (startup.c :: set_fpu_rounding_mode_zero): truncation.
//

import { ASSERT } from "../../core/assert";
import { cIntDivide, toCInt, toCUnsignedInt } from "../../core/cint";
import { f32Add, f32Sub, toFloat32RTZ } from "../../core/float32";
import { KILOMETRE } from "../../core/maths/miscmath";
import type { Vec3d } from "../../core/maths/vec3d";

// C provenance: en_world.h :: struct WORLD_MAP_DATA
export interface WorldMapData {
	sector_side_length: number;
	sector_side_length_mask: number;
	num_map_x_sectors: number;
	num_map_z_sectors: number;
	num_map_sectors: number;
	min_map_x_sector: number;
	min_map_z_sector: number;
	max_map_x_sector: number;
	max_map_z_sector: number;
	min_map_x: number;
	min_map_y: number;
	min_map_z: number;
	mid_map_x: number;
	mid_map_y: number;
	mid_map_z: number;
	max_map_x: number;
	max_map_y: number;
	max_map_z: number;
}

function zeroWorldMap(): WorldMapData {
	return {
		sector_side_length: 0,
		sector_side_length_mask: 0,
		num_map_x_sectors: 0,
		num_map_z_sectors: 0,
		num_map_sectors: 0,
		min_map_x_sector: 0,
		min_map_z_sector: 0,
		max_map_x_sector: 0,
		max_map_z_sector: 0,
		min_map_x: 0,
		min_map_y: 0,
		min_map_z: 0,
		mid_map_x: 0,
		mid_map_y: 0,
		mid_map_z: 0,
		max_map_x: 0,
		max_map_y: 0,
		max_map_z: 0,
	};
}

// C provenance: en_world.c :: world_map_data world_map (static storage: zero)
let world_map: WorldMapData = zeroWorldMap();

export function resetWorldMap(): void {
	world_map = zeroWorldMap();
}

export function getWorldMap(): WorldMapData {
	return world_map;
}

// C provenance: misc/miscell.c :: int_bit_count (unsigned int value)
export function intBitCount(value: number): number {
	let remaining = toCUnsignedInt(value);

	let count = 0;

	while (remaining !== 0) {
		if (remaining % 2 === 1) {
			count++;
		}

		remaining = Math.floor(remaining / 2);
	}

	return count;
}

// C provenance: en_world.c :: set_entity_world_map_size
export function setEntityWorldMapSize(num_map_x_sectors: number, num_map_z_sectors: number, sector_side_length: number): void {
	ASSERT(num_map_x_sectors > 0, "num_map_x_sectors > 0");

	ASSERT(num_map_z_sectors > 0, "num_map_z_sectors > 0");

	ASSERT(intBitCount(sector_side_length) === 1, "int_bit_count (sector_side_length) == 1");

	world_map.sector_side_length = sector_side_length;

	world_map.sector_side_length_mask = sector_side_length - 1;

	world_map.num_map_x_sectors = num_map_x_sectors;
	world_map.num_map_z_sectors = num_map_z_sectors;

	world_map.num_map_sectors = num_map_x_sectors * num_map_z_sectors;

	world_map.min_map_x_sector = 0;
	world_map.min_map_z_sector = 0;

	world_map.max_map_x_sector = num_map_x_sectors - 1;
	world_map.max_map_z_sector = num_map_z_sectors - 1;

	world_map.min_map_x = 0.0;
	world_map.min_map_y = -8000.0;
	world_map.min_map_z = 0.0;

	// (float) (num_map_x_sectors * sector_side_length) - 1.0: the int converted
	// to float, then a double subtraction stored as float (toward zero)
	world_map.max_map_x = f32Add(toFloat32RTZ(num_map_x_sectors * sector_side_length), -1.0);
	world_map.max_map_y = 65535.0;
	world_map.max_map_z = f32Add(toFloat32RTZ(num_map_z_sectors * sector_side_length), -1.0);

	// min + ((max - min) * 0.5): float subtraction, then a double sum (the
	// halving is exact) stored as float
	world_map.mid_map_x = f32Add(world_map.min_map_x, f32Sub(world_map.max_map_x, world_map.min_map_x) * 0.5);
	world_map.mid_map_y = f32Add(world_map.min_map_y, f32Sub(world_map.max_map_y, world_map.min_map_y) * 0.5);
	world_map.mid_map_z = f32Add(world_map.min_map_z, f32Sub(world_map.max_map_z, world_map.min_map_z) * 0.5);
}

// C provenance: en_world.h :: #define point_inside_map_area(POS)
export function pointInsideMapArea(pos: Vec3d): boolean {
	return pos.x >= world_map.min_map_x && pos.x <= world_map.max_map_x && pos.z >= world_map.min_map_z && pos.z <= world_map.max_map_z;
}

// C provenance: en_world.h :: #define point_inside_map_volume(POS)
export function pointInsideMapVolume(pos: Vec3d): boolean {
	return (
		pos.x >= world_map.min_map_x &&
		pos.x <= world_map.max_map_x &&
		pos.y >= world_map.min_map_y &&
		pos.y <= world_map.max_map_y &&
		pos.z >= world_map.min_map_z &&
		pos.z <= world_map.max_map_z
	);
}

// C provenance: en_world.h :: #define MAP_PERIMETER_SIZE (5.0 * KILOMETRE)
export const MAP_PERIMETER_SIZE = 5.0 * KILOMETRE;

//
// C provenance: en_world.c :: bound_position_to_adjusted_map_area
//
// MIN_MAP_X + MAP_PERIMETER_SIZE is a float plus a double: compared in double
// and stored to the float member (toward zero). The map extents are whole
// numbers of metres, so these sums are exact.
//
export function boundPositionToAdjustedMapArea(position: Vec3d): boolean {
	let result = false;

	if (position.x < world_map.min_map_x + MAP_PERIMETER_SIZE) {
		position.x = f32Add(world_map.min_map_x, MAP_PERIMETER_SIZE);
		result = true;
	} else if (position.x > world_map.max_map_x - MAP_PERIMETER_SIZE) {
		position.x = f32Add(world_map.max_map_x, -MAP_PERIMETER_SIZE);
		result = true;
	}

	if (position.z < world_map.min_map_z + MAP_PERIMETER_SIZE) {
		position.z = f32Add(world_map.min_map_z, MAP_PERIMETER_SIZE);
		result = true;
	} else if (position.z > world_map.max_map_z - MAP_PERIMETER_SIZE) {
		position.z = f32Add(world_map.max_map_z, -MAP_PERIMETER_SIZE);
		result = true;
	}

	return result;
}

// C provenance: en_world.c :: bound_position_to_map_area (float members, float bounds)
export function boundPositionToMapArea(position: Vec3d): boolean {
	let result = false;

	if (position.x < world_map.min_map_x) {
		position.x = world_map.min_map_x;
		result = true;
	} else if (position.x > world_map.max_map_x) {
		position.x = world_map.max_map_x;
		result = true;
	}

	if (position.z < world_map.min_map_z) {
		position.z = world_map.min_map_z;
		result = true;
	} else if (position.z > world_map.max_map_z) {
		position.z = world_map.max_map_z;
		result = true;
	}

	return result;
}

// C provenance: en_world.c :: bound_position_to_adjusted_map_volume (as the adjusted area, plus y to the map volume)
export function boundPositionToAdjustedMapVolume(position: Vec3d): boolean {
	let result = false;

	if (position.x < world_map.min_map_x + MAP_PERIMETER_SIZE) {
		position.x = f32Add(world_map.min_map_x, MAP_PERIMETER_SIZE);
		result = true;
	} else if (position.x > world_map.max_map_x - MAP_PERIMETER_SIZE) {
		position.x = f32Add(world_map.max_map_x, -MAP_PERIMETER_SIZE);
		result = true;
	}

	if (position.y < world_map.min_map_y) {
		position.y = world_map.min_map_y;
		result = true;
	} else if (position.y > world_map.max_map_y) {
		position.y = world_map.max_map_y;
		result = true;
	}

	if (position.z < world_map.min_map_z + MAP_PERIMETER_SIZE) {
		position.z = f32Add(world_map.min_map_z, MAP_PERIMETER_SIZE);
		result = true;
	} else if (position.z > world_map.max_map_z - MAP_PERIMETER_SIZE) {
		position.z = f32Add(world_map.max_map_z, -MAP_PERIMETER_SIZE);
		result = true;
	}

	return result;
}

// C provenance: en_world.h :: #define get_x_sector(X_SEC,X)
//   {convert_float_to_int ((X), &(X_SEC)); (X_SEC) /= SECTOR_SIDE_LENGTH;}
export function getXSector(x: number): number {
	return cIntDivide(toCInt(x), world_map.sector_side_length);
}

// C provenance: en_world.h :: #define get_z_sector(Z_SEC,Z)
export function getZSector(z: number): number {
	return cIntDivide(toCInt(z), world_map.sector_side_length);
}
