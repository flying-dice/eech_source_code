//
// eech-core-ts public entry point.
//
// C provenance: entity/system/en_funcs/en_funcs.c :: initialise_entity_functions
//               (ported subset: only the overloads of adopted slices are installed)
//

import type { CampaignPorts } from "./ports";
import { initialiseEntityRuntime, type UnportedMessagePolicy } from "./entity/system/entity";
import { overloadMobileFunctions } from "./entity/mobile/mobile";
import { overloadForceFunctions } from "./entity/special/force/force";
import { overloadGroupFunctions } from "./entity/special/group/group";
import { overloadGuideFunctions } from "./entity/special/guide/guide";
import { overloadKeysiteFunctions } from "./entity/special/keysite/keysite";
import { overloadSessionListFunctions } from "./entity/special/session/session";

export interface CampaignCoreOptions {
	// "throw" in production. Test harnesses may use "record" to observe
	// deliveries to message responses that are not ported yet.
	unportedMessagePolicy?: UnportedMessagePolicy;
}

export function initialiseCampaignCore(ports: CampaignPorts, options: CampaignCoreOptions = {}): void {
	initialiseEntityRuntime(ports, options.unportedMessagePolicy ?? "throw");

	overloadSessionListFunctions();
	overloadForceFunctions();
	overloadKeysiteFunctions();
	overloadGroupFunctions();
	overloadGuideFunctions();
	overloadMobileFunctions();
}

export type { CampaignPorts, EntityReplication, MobilePhysicalState } from "./ports";
export { assessGroupSupplies } from "./entity/special/group/group";

// Campaign state restoration primitives (the state en_pack.c unpacking
// establishes). A CampaignStore port will replace these once session loading
// is ported; until then hosts and tests build state with them.
export { insertLocalEntityIntoParentsChildListRaw } from "./entity/system/en_list";
export { createLocalEntityRaw, setSessionEntityRaw, takeUnportedMessageLog } from "./entity/system/entity";
export { EntitySide, EntitySubTypeGroup, EntitySubTypeKeysite, EntityType, FloatType, ListType } from "./generated/c-enums";
