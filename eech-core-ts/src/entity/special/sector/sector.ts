//
// Sector entities: the campaign's spatial grid.
//
// C provenance: entity/special/sector/sector.c, sector.h, sc_seccreat.c,
//               sc_int.c, sc_list.c, sc_msgs.c
//
// One local SECTOR entity per map cell, created before any client/server
// entity, and a map from cell to entity. A mobile entity is in the
// LIST_TYPE_SECTOR list of the sector under its position.
//
// Ported to the extent the cargo lifecycle reaches: creation of the grid,
// the lookup, and the link / unlink child responses for entities that are
// neither fixed, aircraft nor vehicles. Those three arms (tallest structure
// height from the 3D object bounds, imap defence levels, fog of war and
// FORCE_ENTERED_SECTOR) fail loudly.
//

import { ASSERT, assertNotNullDereference, EechFatalError, EechUndefinedBehaviourError, UnportedBehaviourError } from "../../../core/assert";
import { storeUnsignedBitfield } from "../../../core/cint";
import type { Vec3d } from "../../../core/maths/vec3d";
import { f32Add } from "../../../core/float32";
import { CommsModelType, EntityMessage, EntitySide, EntityType, IntType, ListType } from "../../../generated/c-enums";
import { getCommsModel } from "../../system/comms";
import { createLocalEntity, fnCreateLocalEntity, validateLocalCreateEntityIndex } from "../../system/en_creat";
import { setLocalEntityAttributes, type EntityAttribute } from "../../system/en_attrs";
import { ENTITY_INDEX_DONT_CARE, getFreeEntity } from "../../system/en_heap";
import { overloadEntityListRoot } from "../../system/en_list";
import { messageResponses, type MessageResponseFn } from "../../system/en_msgs";
import { fnGetLocalEntityIntValue, fnSetLocalEntityRawIntValue, getLocalEntityIntValue } from "../../system/en_values";
import { getWorldMap, getXSector, getZSector, pointInsideMapArea } from "../../system/en_world";
import { getLocalEntityData, setLocalEntityData, setLocalEntityType, type Entity } from "../../system/entity";

// C provenance: en_int.h :: NUM_SECTOR_BITS
export const NUM_SECTOR_BITS = 8;

// C provenance: sector.h :: struct SECTOR (ported fields only; memset to 0 on creation)
export interface SectorRaw {
	x_sector: number;
	z_sector: number;
	// float sector_side [NUM_ENTITY_SIDES]
	sector_side: number[];
	// float surface_to_air_defence_level [NUM_ENTITY_SIDES]
	surface_to_air_defence_level: number[];
}

interface SectorMapState {
	// C provenance: sector.c :: entity **entity_sector_map
	entity_sector_map: Record<number, Entity> | undefined;

	// C provenance: sc_seccreat.c :: create_local_sector_entities, static int old_* = -1
	old_min_map_z_sector: number;
	old_max_map_z_sector: number;
	old_num_map_z_sectors: number;
	old_min_map_x_sector: number;
	old_max_map_x_sector: number;
	old_num_map_x_sectors: number;
}

function initialSectorMapState(): SectorMapState {
	return {
		entity_sector_map: undefined,
		old_min_map_z_sector: -1,
		old_max_map_z_sector: -1,
		old_num_map_z_sectors: -1,
		old_min_map_x_sector: -1,
		old_max_map_x_sector: -1,
		old_num_map_x_sectors: -1,
	};
}

let state: SectorMapState = initialSectorMapState();

export function resetSectorMap(): void {
	state = initialSectorMapState();
}

// C provenance: sector.h :: #define get_local_raw_sector_entity(X_SEC,Z_SEC) (release build)
export function getLocalRawSectorEntity(x_sec: number, z_sec: number): Entity | undefined {
	const map = state.entity_sector_map;

	// C indexes entity_sector_map without a check
	assertNotNullDereference(map, "entity_sector_map [(X_SEC) + ((Z_SEC) * (NUM_MAP_X_SECTORS))]");

	const index = x_sec + z_sec * getWorldMap().num_map_x_sectors;

	// C reads past the array for a cell outside the map (e.g. a route node off
	// the map in task.c :: assess_task_difficulty)
	if (index < 0 || index >= getWorldMap().num_map_sectors) {
		throw new EechUndefinedBehaviourError("entity_sector_map read outside the map");
	}

	return map[index];
}

// C provenance: sc_seccreat.c :: create_local_sector_entities
export function createLocalSectorEntities(): void {
	const world_map = getWorldMap();

	// First time setup
	if (state.old_min_map_z_sector === -1) {
		state.old_min_map_z_sector = world_map.min_map_z_sector;
		state.old_max_map_z_sector = world_map.max_map_z_sector;
		state.old_num_map_z_sectors = world_map.num_map_z_sectors;
		state.old_min_map_x_sector = world_map.min_map_x_sector;
		state.old_max_map_x_sector = world_map.max_map_x_sector;
		state.old_num_map_x_sectors = world_map.num_map_x_sectors;
	}

	// Deinitialise sectors (must be the 'old' map size)
	const old_map = state.entity_sector_map;

	if (old_map !== undefined) {
		for (let z_sec = state.old_min_map_z_sector; z_sec <= state.old_max_map_z_sector; z_sec++) {
			for (let x_sec = state.old_min_map_x_sector; x_sec <= state.old_max_map_x_sector; x_sec++) {
				if (old_map[x_sec + z_sec * state.old_num_map_x_sectors].type !== EntityType.ENTITY_TYPE_UNKNOWN) {
					throw new EechFatalError("SC_CREAT: uninitialised sector entity");
				}
			}
		}
	}

	// Store new map size
	state.old_min_map_z_sector = world_map.min_map_z_sector;
	state.old_max_map_z_sector = world_map.max_map_z_sector;
	state.old_num_map_z_sectors = world_map.num_map_z_sectors;
	state.old_min_map_x_sector = world_map.min_map_x_sector;
	state.old_max_map_x_sector = world_map.max_map_x_sector;
	state.old_num_map_x_sectors = world_map.num_map_x_sectors;

	const map: Record<number, Entity> = {};

	state.entity_sector_map = map;

	for (let z_sec = world_map.min_map_z_sector; z_sec <= world_map.max_map_z_sector; z_sec++) {
		for (let x_sec = world_map.min_map_x_sector; x_sec <= world_map.max_map_x_sector; x_sec++) {
			map[x_sec + z_sec * world_map.num_map_x_sectors] = createLocalEntity(EntityType.ENTITY_TYPE_SECTOR, ENTITY_INDEX_DONT_CARE, [
				{ kind: "int_value", type: IntType.INT_TYPE_X_SECTOR, value: x_sec },
				{ kind: "int_value", type: IntType.INT_TYPE_Z_SECTOR, value: z_sec },
			]);
		}
	}
}

// C provenance: sc_seccreat.c :: create_local
function createLocal(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateLocalCreateEntityIndex(index);

	const en = getFreeEntity(index);

	if (en) {
		setLocalEntityType(en, type);

		const raw: SectorRaw = { x_sector: 0, z_sector: 0, sector_side: [0.0, 0.0, 0.0], surface_to_air_defence_level: [0.0, 0.0, 0.0] };

		setLocalEntityData(en, raw);

		setLocalEntityAttributes(en, attributes);
	}

	return en;
}

// C provenance: sector.c :: get_local_sector_entity
export function getLocalSectorEntity(pos: Vec3d): Entity {
	if (!pointInsideMapArea(pos)) {
		throw new EechFatalError("Position off map: (x = %f, z = %f)", `Position off map: (x = ${pos.x}, z = ${pos.z})`);
	}

	const x_sec = getXSector(pos.x);
	const z_sec = getZSector(pos.z);

	const en = getLocalRawSectorEntity(x_sec, z_sec);

	ASSERT(en !== undefined, "en");

	return en;
}

//
// C provenance: sector.c :: get_local_sector_entity_enemy_defence_level (static)
//
// The sum of every side's level but `side`'s, neutral excluded:
// loop = ENTITY_SIDE_NEUTRAL; while (++ loop < NUM_ENTITY_SIDES). Float sums.
//
function getLocalSectorEntityEnemyDefenceLevel(array: number[], side: EntitySide): number {
	let defence_level = 0;

	for (let loop = EntitySide.ENTITY_SIDE_NEUTRAL + 1; loop < EntitySide.NUM_ENTITY_SIDES; loop++) {
		if (side === loop) {
			continue;
		}

		defence_level = f32Add(defence_level, array[loop]);
	}

	return defence_level;
}

// C provenance: sector.c :: get_local_sector_entity_enemy_surface_to_air_defence_level
export function getLocalSectorEntityEnemySurfaceToAirDefenceLevel(sector_en: Entity, side: EntitySide): number {
	return getLocalSectorEntityEnemyDefenceLevel(getLocalEntityData<SectorRaw>(sector_en).surface_to_air_defence_level, side);
}

// C provenance: sector.c :: add_mobile_values_to_sector
export function addMobileValuesToSector(_sector_en: Entity, mobile_en: Entity): void {
	if (getCommsModel() !== CommsModelType.COMMS_MODEL_SERVER) {
		return;
	}

	if (getLocalEntityIntValue(mobile_en, IntType.INT_TYPE_ALIVE) !== 0) {
		if (getLocalEntityIntValue(mobile_en, IntType.INT_TYPE_IDENTIFY_VEHICLE) !== 0) {
			throw new UnportedBehaviourError("sector.c :: add_mobile_values_to_sector (vehicle) -> update_imap_surface_to_*_defence_level");
		}
	}
}

// C provenance: sector.c :: remove_mobile_values_from_sector
export function removeMobileValuesFromSector(_sector_en: Entity, mobile_en: Entity | undefined): void {
	ASSERT(mobile_en !== undefined, "mobile_en");

	if (getCommsModel() !== CommsModelType.COMMS_MODEL_SERVER) {
		return;
	}

	if (getLocalEntityIntValue(mobile_en, IntType.INT_TYPE_ALIVE) !== 0) {
		if (getLocalEntityIntValue(mobile_en, IntType.INT_TYPE_IDENTIFY_VEHICLE) !== 0) {
			throw new UnportedBehaviourError("sector.c :: remove_mobile_values_from_sector (vehicle) -> update_imap_surface_to_*_defence_level");
		}
	}
}

// C provenance: sc_msgs.c :: response_to_link_child
const responseToLinkChild: MessageResponseFn = (_message, receiver, sender, args) => {
	const list_type = args[0] as ListType;

	if (list_type === ListType.LIST_TYPE_SECTOR) {
		// C reads the sender's values without a check
		assertNotNullDereference(sender, "get_local_entity_int_value (sender, INT_TYPE_IDENTIFY_FIXED)");

		// check for tallest fixed structure
		if (getLocalEntityIntValue(sender, IntType.INT_TYPE_IDENTIFY_FIXED) !== 0) {
			throw new UnportedBehaviourError("sc_msgs.c :: response_to_link_child (fixed entity) -> get_object_3d_bounding_box (Slice 4)");
		}

		// set defence levels / activity array
		addMobileValuesToSector(receiver, sender);

		// set side awareness + notify force
		if (getLocalEntityIntValue(sender, IntType.INT_TYPE_IDENTIFY_AIRCRAFT) !== 0 || getLocalEntityIntValue(sender, IntType.INT_TYPE_IDENTIFY_VEHICLE) !== 0) {
			throw new UnportedBehaviourError("sc_msgs.c :: response_to_link_child (aircraft or vehicle) -> fog of war, FORCE_ENTERED_SECTOR");
		}
	}

	return 1;
};

// C provenance: sc_msgs.c :: response_to_unlink_child
const responseToUnlinkChild: MessageResponseFn = (_message, receiver, sender, args) => {
	if ((args[0] as ListType) === ListType.LIST_TYPE_SECTOR) {
		removeMobileValuesFromSector(receiver, sender);
	}

	return 1;
};

export function overloadSectorFunctions(): void {
	const SECTOR = EntityType.ENTITY_TYPE_SECTOR;

	// C provenance: sc_seccreat.c :: overload_sector_create_functions
	fnCreateLocalEntity.overload(SECTOR, 0, createLocal);

	// C provenance: sc_list.c :: LIST_TYPE_SECTOR_ROOT, LIST_TYPE_SECTOR_TASK_ROOT, LIST_TYPE_SPECIAL_EFFECT_ROOT
	overloadEntityListRoot(SECTOR, "sector_root", [ListType.LIST_TYPE_SECTOR]);
	overloadEntityListRoot(SECTOR, "sector_task_root", [ListType.LIST_TYPE_SECTOR_TASK]);
	overloadEntityListRoot(SECTOR, "special_effect_root", [ListType.LIST_TYPE_SPECIAL_EFFECT]);

	// C provenance: sc_int.c :: set_local_int_value (raw), get_local_int_value
	//   raw->x_sector / z_sector are `unsigned int : NUM_SECTOR_BITS` bit-fields
	fnSetLocalEntityRawIntValue.overload(SECTOR, IntType.INT_TYPE_X_SECTOR, (en, _type, value) => {
		getLocalEntityData<SectorRaw>(en).x_sector = storeUnsignedBitfield(value, NUM_SECTOR_BITS);
	});
	fnSetLocalEntityRawIntValue.overload(SECTOR, IntType.INT_TYPE_Z_SECTOR, (en, _type, value) => {
		getLocalEntityData<SectorRaw>(en).z_sector = storeUnsignedBitfield(value, NUM_SECTOR_BITS);
	});
	fnGetLocalEntityIntValue.overload(SECTOR, IntType.INT_TYPE_X_SECTOR, (en) => getLocalEntityData<SectorRaw>(en).x_sector);
	fnGetLocalEntityIntValue.overload(SECTOR, IntType.INT_TYPE_Z_SECTOR, (en) => getLocalEntityData<SectorRaw>(en).z_sector);

	// C provenance: sc_int.c :: get_local_int_value (INT_TYPE_SECTOR_SIDE): RED unless blue's presence is greater
	fnGetLocalEntityIntValue.overload(SECTOR, IntType.INT_TYPE_SECTOR_SIDE, (en) => {
		const raw = getLocalEntityData<SectorRaw>(en);

		return raw.sector_side[EntitySide.ENTITY_SIDE_BLUE_FORCE] > raw.sector_side[EntitySide.ENTITY_SIDE_RED_FORCE]
			? EntitySide.ENTITY_SIDE_BLUE_FORCE
			: EntitySide.ENTITY_SIDE_RED_FORCE;
	});

	// C provenance: sc_msgs.c :: overload_sector_message_responses
	messageResponses.overload(SECTOR, EntityMessage.ENTITY_MESSAGE_LINK_CHILD, responseToLinkChild);
	messageResponses.overload(SECTOR, EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, responseToUnlinkChild);
}
