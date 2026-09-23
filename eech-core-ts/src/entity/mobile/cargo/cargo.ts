//
// Cargo entities: supply crates at keysites and in transit.
//
// C provenance: entity/mobile/cargo/cargo.h, cg_creat.c, cg_dstry.c, cg_list.c,
//               cg_funcs.c, cg_msgs.c
//
// A cargo is a mobile entity whose position is campaign state (set by its
// creation attributes), not a physical simulation. Creation links it into its
// keysite's LIST_TYPE_CARGO (when a parent attribute is given) and always into
// the sector under its position. Destruction unlinks it everywhere and frees
// its heap entry.
//
// Not ported: kill, movement (cg_move.c), update, draw, pack, and the int
// values of cg_int.c (no ported reader).
//

import { toFloat32RTZ } from "../../../core/float32";
import { CommsModelType, EntityType, ListType } from "../../../generated/c-enums";
import { overloadAircraftLinkParentResponses } from "../aircraft/ac_msgs";
import { ENTITY_SIDE_UNINITIALISED, ENTITY_SUB_TYPE_UNINITIALISED, overloadMobileRawStateFunctions, type MobileRaw } from "../mobile";
import { destroyClientServerSoundEffects } from "../../special/effect/soundeff";
import { getLocalSectorEntity } from "../../special/sector/sector";
import { replicatedEntityAttributes, setLocalEntityAttributes, type EntityAttribute } from "../../system/en_attrs";
import { fnCreateClientServerEntity, fnCreateLocalEntity, validateLocalCreateEntityIndex, validateRemoteCreateEntityIndex } from "../../system/en_creat";
import { destroyClientServerEntity, destroyLocalEntity, fnDestroyClientServerEntity, fnDestroyClientServerEntityFamily, fnDestroyLocalEntity } from "../../system/en_dstry";
import { getFreeEntity, setFreeEntity } from "../../system/en_heap";
import {
	deleteLocalEntityFromParentsChildList,
	getLocalEntityParent,
	insertLocalEntityIntoParentsChildList,
	overloadEntityListLink,
	overloadEntityListRoot,
	unlinkLocalEntityChildren,
} from "../../system/en_list";
import { getWorldMap } from "../../system/en_world";
import { getCampaignPorts, setLocalEntityData, setLocalEntityType, type Entity } from "../../system/entity";

// C provenance: cargo.h :: struct CARGO (ported fields only)
export interface CargoRaw {
	mob: MobileRaw;
}

// C provenance: cg_creat.c :: create_local
function createLocal(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateLocalCreateEntityIndex(index);

	const en = getFreeEntity(index);

	if (en) {
		setLocalEntityType(en, type);

		// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES (after memset (raw, 0, sizeof (cargo)))
		const world_map = getWorldMap();

		const raw: CargoRaw = {
			mob: {
				sub_type: ENTITY_SUB_TYPE_UNINITIALISED,
				position: { x: toFloat32RTZ(world_map.mid_map_x), y: toFloat32RTZ(world_map.mid_map_y), z: toFloat32RTZ(world_map.mid_map_z) },
				alive: 1,
				side: ENTITY_SIDE_UNINITIALISED,
			},
		};

		setLocalEntityData(en, raw);

		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		setLocalEntityAttributes(en, attributes);

		// LINK INTO SYSTEM
		const parent = getLocalEntityParent(en, ListType.LIST_TYPE_CARGO);

		if (parent) {
			insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_CARGO, parent, undefined);
		}

		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_SECTOR, getLocalSectorEntity(raw.mob.position), undefined);
	}

	return en;
}

// C provenance: cg_creat.c :: create_remote
function createRemote(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateRemoteCreateEntityIndex(index);

	getCampaignPorts().entityReplication.transmitEntityCreate(type, index, replicatedEntityAttributes(attributes));

	return undefined;
}

// C provenance: cg_creat.c :: create_server
function createServer(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	const en = createLocal(type, index, attributes);

	if (en) {
		createRemote(type, en.index, attributes);
	}

	return en;
}

// C provenance: cg_dstry.c :: destroy_local
function destroyLocal(en: Entity): void {
	// cargo
	unlinkLocalEntityChildren(en, ListType.LIST_TYPE_TASK_DEPENDENT);
	deleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_CARGO);
	deleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_MOVEMENT_DEPENDENT);

	// mobile
	unlinkLocalEntityChildren(en, ListType.LIST_TYPE_SPECIAL_EFFECT);
	unlinkLocalEntityChildren(en, ListType.LIST_TYPE_TARGET);
	deleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_PADLOCK);
	deleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_SECTOR);
	deleteLocalEntityFromParentsChildList(en, ListType.LIST_TYPE_UPDATE);

	// FREE ENTITY DATA
	setFreeEntity(en);
}

// C provenance: cg_dstry.c :: destroy_remote
function destroyRemote(en: Entity): void {
	getCampaignPorts().entityReplication.transmitEntityDestroy(en.index);
}

// C provenance: cg_dstry.c :: destroy_server (destroy remote entity first, keeping local entity valid)
function destroyServer(en: Entity): void {
	destroyRemote(en);

	destroyLocalEntity(en);
}

// C provenance: cg_dstry.c :: destroy_server_family
function destroyServerFamily(en: Entity): void {
	destroyClientServerSoundEffects(en);

	destroyClientServerEntity(en);
}

// C provenance: cg_funcs.c :: overload_cargo_functions (ported subset), cg_msgs.c
export function overloadCargoFunctions(): void {
	const CARGO = EntityType.ENTITY_TYPE_CARGO;
	const SERVER = CommsModelType.COMMS_MODEL_SERVER;

	overloadMobileRawStateFunctions(CARGO);

	// C provenance: cg_creat.c :: overload_cargo_create_functions
	fnCreateLocalEntity.overload(CARGO, 0, createLocal);
	fnCreateClientServerEntity.overload(CARGO, SERVER, createServer);

	// C provenance: cg_dstry.c :: overload_cargo_destroy_functions
	fnDestroyLocalEntity.overload(CARGO, 0, destroyLocal);
	fnDestroyClientServerEntity.overload(CARGO, SERVER, destroyServer);
	fnDestroyClientServerEntityFamily.overload(CARGO, SERVER, destroyServerFamily);

	// C provenance: cg_list.c :: LIST_TYPE_TASK_DEPENDENT_ROOT, LIST_TYPE_CARGO_LINK, LIST_TYPE_MOVEMENT_DEPENDENT_LINK
	overloadEntityListRoot(CARGO, "task_dependent_root", [ListType.LIST_TYPE_TASK_DEPENDENT]);
	overloadEntityListLink(CARGO, "cargo_link", [ListType.LIST_TYPE_CARGO]);
	overloadEntityListLink(CARGO, "movement_dependent_link", [ListType.LIST_TYPE_MOVEMENT_DEPENDENT]);

	// C provenance: cg_msgs.c :: overload_cargo_message_responses -> overload_aircraft_message_responses
	overloadAircraftLinkParentResponses(CARGO);
}
