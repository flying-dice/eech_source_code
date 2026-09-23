//
// eech-core-ts public entry point.
//
// C provenance: entity/system/en_funcs/en_funcs.c :: initialise_entity_functions
//               (ported subset: only the overloads of adopted slices are installed)
//

import type { CampaignPorts } from "./ports";
import { DEFAULT_ENTITY_UPDATE_FRAME_RATE, setCommandLineEntityUpdateFrameRate } from "./core/cmndline";
import { resetDeltaTime, setDeltaTimeFrom } from "./core/time";
import { resetGameStatus } from "./core/game-status";
import { getCampaignPorts } from "./entity/system/entity";
import { overloadUpdateFunctions, resetUpdateEntity } from "./entity/special/update/update";
import { initialiseEntityRuntime } from "./entity/system/entity";
import { resetCreateSupplyTaskObserver } from "./ai/taskgen/taskgen";
import { initialiseGroupTaskArray } from "./ai/highlevl/suitable";
import { resetGameType } from "./core/game-type";
import { resetEntityCommsTransmission } from "./entity/system/en_comms";
import { overloadTaskCreateFunctions } from "./entity/special/task/ts_creat";
import { overloadMobileFunctions } from "./entity/mobile/mobile";
import { overloadForceFunctions } from "./entity/special/force/force";
import { overloadForceMessageResponses } from "./entity/special/force/fc_msgs";
import { overloadTaskFunctions } from "./entity/special/task/task";
import { overloadWaypointFunctions } from "./entity/special/waypoint/waypoint";
import { overloadWaypointCreateFunctions } from "./entity/special/waypoint/wp_creat";
import { initialiseInverseSquareRootTable } from "./core/maths/invsqrt";
import { overloadGroupFunctions } from "./entity/special/group/group";
import { overloadGuideFunctions } from "./entity/special/guide/guide";
import { overloadPilotFunctions } from "./entity/special/pilot/pilot";
import { overloadKeysiteFunctions } from "./entity/special/keysite/keysite";
import { overloadSessionListFunctions } from "./entity/special/session/session";
import { overloadCargoFunctions } from "./entity/mobile/cargo/cargo";
import { overloadSectorFunctions, resetSectorMap } from "./entity/special/sector/sector";
import { DEFAULT_NUMBER_OF_ENTITIES, initialiseEntityHeap } from "./entity/system/en_heap";
import { overloadUnknownEntityDestroyFunctions } from "./entity/system/en_dstry";
import { resetWorldMap } from "./entity/system/en_world";

export interface CampaignCoreOptions {
	// EECH.INI "entity update frame rate" (cmndline.c default 2)
	entityUpdateFrameRate?: number;

	// size of the entity heap (init.c :: initialise_entity_system, 125000)
	numberOfEntities?: number;
}

export function initialiseCampaignCore(ports: CampaignPorts, options: CampaignCoreOptions = {}): void {
	initialiseEntityRuntime(ports);
	resetCreateSupplyTaskObserver();
	resetEntityCommsTransmission();
	initialiseEntityHeap(options.numberOfEntities ?? DEFAULT_NUMBER_OF_ENTITIES);
	resetWorldMap();
	resetSectorMap();

	resetDeltaTime();
	resetGameStatus();
	resetGameType();
	resetUpdateEntity();
	setCommandLineEntityUpdateFrameRate(options.entityUpdateFrameRate ?? DEFAULT_ENTITY_UPDATE_FRAME_RATE);

	overloadSessionListFunctions();
	overloadForceFunctions();
	overloadForceMessageResponses();
	overloadKeysiteFunctions();
	overloadGroupFunctions();
	overloadGuideFunctions();
	overloadMobileFunctions();
	overloadUpdateFunctions();
	overloadSectorFunctions();
	overloadCargoFunctions();
	overloadTaskFunctions();
	overloadTaskCreateFunctions();
	overloadWaypointFunctions();
	overloadWaypointCreateFunctions();
	overloadPilotFunctions();
	overloadUnknownEntityDestroyFunctions();

	// C provenance: highlevl.c :: initialise_highlevel_ai -> initialise_group_task_array
	initialiseGroupTaskArray();

	// C provenance: modules/maths/initmath.c :: initialise_maths_library -> initialise_inverse_square_root_table (at start-up)
	initialiseInverseSquareRootTable();
}

//
// Host frame boundary. EECH's flight loop (flight.c) measures the frame
// (set_delta_time) and then calls update_client_server_entities () once per
// time-acceleration step. setDeltaTime () takes the measurement from the Clock
// port; the host decides how many times to call updateClientServerEntities ().
//
export function setDeltaTime(): void {
	setDeltaTimeFrom(getCampaignPorts().clock);
}

export type {
	CampaignEvents,
	CampaignPorts,
	Clock,
	EntityReplication,
	MobilePhysicalState,
	Object3DBounds,
	Object3DMetadata,
	ReplicatedEntityAttribute,
	ReplicatedTaskRoute,
	ReplicatedWaypointRoute,
	RoadNetwork,
	TerrainElevation,
} from "./ports";
export { assessGroupSupplies } from "./entity/special/group/group";
export { setUpdateEntity, updateClientServerEntities } from "./entity/special/update/update";
export { setClientServerEntityFloatValue } from "./entity/system/en_values";

// Keysite cargo (slice 4) and the host's game status
export { updateKeysiteCargo } from "./entity/special/keysite/keysite";
export { getGameStatus, setGameStatus } from "./core/game-status";

// Supply task construction (slice 5b) and the session state it reads
export { createSupplyTask } from "./ai/taskgen/taskgen";
export { getGameType, setGameType } from "./core/game-type";
export { setEntityCommsTransmission } from "./entity/system/en_comms";
export { CARGO_AMMO_SIZE, CARGO_FUEL_SIZE, OBJECT_3D_SINGLE_CRATE } from "./generated/c-constants";

// Entity lifecycle and the world map
export type { EntityAttribute } from "./entity/system/en_attrs";
export { createClientServerEntity } from "./entity/system/en_creat";
export { destroyClientServerEntityFamily } from "./entity/system/en_dstry";
export { ENTITY_INDEX_DONT_CARE } from "./entity/system/en_heap";
export { setEntityWorldMapSize } from "./entity/system/en_world";
export { createLocalSectorEntities } from "./entity/special/sector/sector";

// Campaign state restoration primitives (the state en_pack.c unpacking
// establishes). A CampaignStore port will replace these once session loading
// is ported; until then hosts and tests build state with them.
export { insertLocalEntityIntoParentsChildListRaw } from "./entity/system/en_list";
export { createLocalEntityRaw } from "./entity/system/en_heap";
export { setSessionEntityRaw } from "./entity/system/entity";
export { EntitySide, EntitySubTypeCargo, EntitySubTypeGroup, EntitySubTypeKeysite, EntitySubTypeTask, EntityType, FloatType, GameStatusType, IntType, ListType, MovementType, TaskStateType, Vec3dType } from "./generated/c-enums";
export { EntitySubTypeWaypoint, GameType, KeysiteUsableState } from "./generated/c-enums";
