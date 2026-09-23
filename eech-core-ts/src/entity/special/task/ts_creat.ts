//
// Task entity creation.
//
// C provenance: entity/special/task/ts_creat.c
//
// A new task is on the update list, on its objective's task dependent list
// (when a LIST_TYPE_TASK_DEPENDENT parent attribute is given) and on the task
// list of its state under a given LIST_TYPE_*_TASK parent (none of the
// attributes create_task passes). The client model is not ported.
//

import { CommsModelType, EntityType, ListType, TaskStateType } from "../../../generated/c-enums";
import { ENTITY_SIDE_UNINITIALISED } from "../../mobile/mobile";
import { replicatedEntityAttributes, setLocalEntityAttributes, type EntityAttribute } from "../../system/en_attrs";
import { transmitEntityCreate } from "../../system/en_comms";
import { fnCreateClientServerEntity, fnCreateLocalEntity, validateLocalCreateEntityIndex, validateRemoteCreateEntityIndex } from "../../system/en_creat";
import { getFreeEntity } from "../../system/en_heap";
import { getLocalEntityParent, insertLocalEntityIntoParentsChildList } from "../../system/en_list";
import { setLocalEntityData, setLocalEntityType, type Entity } from "../../system/entity";
import { getUpdateEntity } from "../update/update";
import { clearedTaskRaw, getLocalTaskListType } from "./task";

// C provenance: ts_creat.c :: create_local
function createLocal(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateLocalCreateEntityIndex(index);

	const en = getFreeEntity(index);

	if (en) {
		setLocalEntityType(en, type);

		// INITIALISE ALL ENTITY DATA TO 'WORKING' DEFAULT VALUES (after memset (raw, 0, sizeof (task)))
		const raw = clearedTaskRaw();

		// raw->side = ENTITY_SIDE_UNINITIALISED: 3 fits the 2-bit field
		raw.side = ENTITY_SIDE_UNINITIALISED;
		raw.task_state = TaskStateType.TASK_STATE_UNASSIGNED;

		setLocalEntityData(en, raw);

		// OVERWRITE DEFAULT VALUES WITH GIVEN ATTRIBUTES
		setLocalEntityAttributes(en, attributes);

		// LINK INTO SYSTEM
		insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_UPDATE, getUpdateEntity(), undefined);

		const objective = getLocalEntityParent(en, ListType.LIST_TYPE_TASK_DEPENDENT);

		if (objective) {
			insertLocalEntityIntoParentsChildList(en, ListType.LIST_TYPE_TASK_DEPENDENT, objective, undefined);
		}

		// raw->task_link.parent
		const keysite = getLocalEntityParent(en, ListType.LIST_TYPE_UNASSIGNED_TASK);

		if (keysite) {
			insertLocalEntityIntoParentsChildList(en, getLocalTaskListType(en), keysite, undefined);
		}
	}

	return en;
}

// C provenance: ts_creat.c :: create_remote
function createRemote(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	validateRemoteCreateEntityIndex(index);

	transmitEntityCreate(type, index, replicatedEntityAttributes(attributes));

	return undefined;
}

// C provenance: ts_creat.c :: create_server
function createServer(type: EntityType, index: number, attributes: EntityAttribute[]): Entity | undefined {
	const en = createLocal(type, index, attributes);

	if (en) {
		createRemote(type, en.index, attributes);
	}

	return en;
}

// C provenance: ts_creat.c :: overload_task_create_functions (server model)
export function overloadTaskCreateFunctions(): void {
	fnCreateLocalEntity.overload(EntityType.ENTITY_TYPE_TASK, 0, createLocal);
	fnCreateClientServerEntity.overload(EntityType.ENTITY_TYPE_TASK, CommsModelType.COMMS_MODEL_SERVER, createServer);
}
