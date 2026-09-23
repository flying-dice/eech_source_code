//
// eech-core-ts public entry point.
//
// C provenance: entity/system/en_funcs/en_funcs.c :: initialise_entity_functions
//               (ported subset: only the overloads of adopted slices are installed)
//

import type { CampaignPorts } from "./ports";
import { DEFAULT_ENTITY_UPDATE_FRAME_RATE, setCommandLineEntityUpdateFrameRate } from "./core/cmndline";
import { resetDeltaTime, setDeltaTimeFrom } from "./core/time";
import { getCampaignPorts } from "./entity/system/entity";
import { overloadUpdateFunctions, resetUpdateEntity } from "./entity/special/update/update";
import { initialiseEntityRuntime, type UnportedMessagePolicy } from "./entity/system/entity";
import { overloadMobileFunctions } from "./entity/mobile/mobile";
import { overloadForceFunctions } from "./entity/special/force/force";
import { overloadGroupFunctions } from "./entity/special/group/group";
import { overloadGuideFunctions } from "./entity/special/guide/guide";
import { overloadKeysiteFunctions } from "./entity/special/keysite/keysite";
import { overloadSessionListFunctions } from "./entity/special/session/session";
import { overloadCargoFunctions } from "./entity/mobile/cargo/cargo";
import { overloadSectorFunctions, resetSectorMap } from "./entity/special/sector/sector";
import { DEFAULT_NUMBER_OF_ENTITIES, initialiseEntityHeap } from "./entity/system/en_heap";
import { overloadUnknownEntityDestroyFunctions } from "./entity/system/en_dstry";
import { resetWorldMap } from "./entity/system/en_world";

export interface CampaignCoreOptions {
	// "throw" in production. Test harnesses may use "record" to observe
	// deliveries to message responses that are not ported yet.
	unportedMessagePolicy?: UnportedMessagePolicy;

	// EECH.INI "entity update frame rate" (cmndline.c default 2)
	entityUpdateFrameRate?: number;

	// size of the entity heap (init.c :: initialise_entity_system, 125000)
	numberOfEntities?: number;
}

export function initialiseCampaignCore(ports: CampaignPorts, options: CampaignCoreOptions = {}): void {
	initialiseEntityRuntime(ports, options.unportedMessagePolicy ?? "throw");
	initialiseEntityHeap(options.numberOfEntities ?? DEFAULT_NUMBER_OF_ENTITIES);
	resetWorldMap();
	resetSectorMap();

	resetDeltaTime();
	resetUpdateEntity();
	setCommandLineEntityUpdateFrameRate(options.entityUpdateFrameRate ?? DEFAULT_ENTITY_UPDATE_FRAME_RATE);

	overloadSessionListFunctions();
	overloadForceFunctions();
	overloadKeysiteFunctions();
	overloadGroupFunctions();
	overloadGuideFunctions();
	overloadMobileFunctions();
	overloadUpdateFunctions();
	overloadSectorFunctions();
	overloadCargoFunctions();
	overloadUnknownEntityDestroyFunctions();
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

export type { CampaignPorts, Clock, EntityReplication, MobilePhysicalState, ReplicatedEntityAttribute } from "./ports";
export { assessGroupSupplies } from "./entity/special/group/group";
export { setUpdateEntity, updateClientServerEntities } from "./entity/special/update/update";
export { setClientServerEntityFloatValue } from "./entity/system/en_values";

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
export { setSessionEntityRaw, takeUnportedMessageLog } from "./entity/system/entity";
export { EntitySide, EntitySubTypeGroup, EntitySubTypeKeysite, EntityType, FloatType, IntType, ListType, Vec3dType } from "./generated/c-enums";
