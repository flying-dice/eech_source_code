//
// The entity heap.
//
// C provenance: entity/system/en_main/en_heap.c, en_heap.h
//
// A fixed array of entity records threaded into a free list and a used list
// through `succ` / `pred`. Allocation takes the head of the free list and
// pushes the entity at the head of the used list; freeing pushes it back at
// the head of the free list, so the most recently freed index is reused first.
//
// get_free_entity with a specific index (clients and saved games choose the
// index) takes that entry out of the middle of the free list; creating
// entities on a client stays unported at the create tables.
//
// Not ported:
//   - the downwash local-only heap (command_line_downwash,
//     ENTITY_INDEX_CREATE_LOCAL), off by default;
//   - pack_entity_safe_index and friends (saved games and network messages).
//

import { ASSERT, EechFatalError } from "../../core/assert";
import { EntityType } from "../../generated/c-enums";
import { getEntityHeapState, getLocalEntityPtr, getLocalEntitySafePtr, resetEntityRecords, setLocalEntityData, setLocalEntityType, type Entity } from "./entity";

// C provenance: en_heap.h
export const ENTITY_INDEX_DONT_CARE = -1;

export const MAX_NUM_ENTITIES = 131072 * 4;

// C provenance: init.c :: initialise_entity_system (125000), without downwash
export const DEFAULT_NUMBER_OF_ENTITIES = 125000;

// C provenance: en_heap.c :: initialise_entity_heap, reset_entity_heap
export function initialiseEntityHeap(num_entities: number): void {
	ASSERT(num_entities > 0 && num_entities < MAX_NUM_ENTITIES, "(num_entities > 0) && (num_entities < MAX_NUM_ENTITIES)");

	resetEntityRecords(num_entities);

	const heap = getEntityHeapState();

	heap.firstFreeEntity = 0;
	heap.firstUsedEntity = -1;
}

// C provenance: en_heap.c :: get_free_entity
export function getFreeEntity(index: number): Entity | undefined {
	if (index !== ENTITY_INDEX_DONT_CARE) {
		return getFreeEntityAt(index);
	}

	const heap = getEntityHeapState();

	const en = getLocalEntitySafePtr(heap.firstFreeEntity);

	if (en) {
		// unlink entity from start of free list
		heap.firstFreeEntity = en.succ;

		const succ = getLocalEntitySafePtr(en.succ);

		if (succ) {
			succ.pred = -1;
		}

		// insert entity into start of used list (en->pred already set to NULL)
		en.succ = heap.firstUsedEntity;

		const used = getLocalEntitySafePtr(en.succ);

		if (used) {
			used.pred = en.index;
		}

		heap.firstUsedEntity = en.index;
	}

	// else: debug_colour_log ("WARNING! Failed to get a free entity") and NULL

	return en;
}

// C provenance: en_heap.c :: get_free_entity, the `index != ENTITY_INDEX_DONT_CARE` arm
function getFreeEntityAt(index: number): Entity {
	const heap = getEntityHeapState();

	// ASSERT ((index >= 0) && (index < number_of_entities)), in get_local_entity_ptr
	const en = getLocalEntityPtr(index);

	if (en.type !== EntityType.ENTITY_TYPE_UNKNOWN) {
		throw new EechFatalError("Entity already in use: %s (index = %d)", `Entity already in use: ${EntityType[en.type]} (index = ${index})`);
	}

	// unlink entity from free list
	const pred = getLocalEntitySafePtr(en.pred);

	if (pred) {
		pred.succ = en.succ;
	} else {
		heap.firstFreeEntity = en.succ;
	}

	const succ = getLocalEntitySafePtr(en.succ);

	if (succ) {
		succ.pred = en.pred;
	}

	// insert entity into start of used list
	en.succ = heap.firstUsedEntity;

	const used = getLocalEntitySafePtr(en.succ);

	if (used) {
		used.pred = en.index;
	}

	en.pred = -1;

	heap.firstUsedEntity = en.index;

	return en;
}

// C provenance: en_heap.c :: set_free_entity
export function setFreeEntity(en: Entity): void {
	const heap = getEntityHeapState();

	en.type = EntityType.ENTITY_TYPE_UNKNOWN;
	en.data = undefined;
	en.roots = {};
	en.links = {};

	// unlink entity from used list
	const pred = getLocalEntitySafePtr(en.pred);

	if (pred) {
		pred.succ = en.succ;
	} else {
		heap.firstUsedEntity = en.succ;
	}

	const succ = getLocalEntitySafePtr(en.succ);

	if (succ) {
		succ.pred = en.pred;
	}

	// link entity into start of free list
	en.succ = heap.firstFreeEntity;

	const free = getLocalEntitySafePtr(en.succ);

	if (free) {
		free.pred = en.index;
	}

	en.pred = -1;

	heap.firstFreeEntity = en.index;
}

// C provenance: en_heap.h :: first_free_entity / get_local_entity_list () (first_used_entity)
export function getFirstFreeEntity(): Entity | undefined {
	return getLocalEntitySafePtr(getEntityHeapState().firstFreeEntity);
}

export function getLocalEntityList(): Entity | undefined {
	return getLocalEntitySafePtr(getEntityHeapState().firstUsedEntity);
}

// C provenance: en_heap.h :: en->succ (heap list successor)
export function getLocalEntitySucc(en: Entity): Entity | undefined {
	return getLocalEntitySafePtr(en.succ);
}

export { getLocalEntityPtr };

//
// Establishes an entity the way loading a saved campaign does: the next free
// heap entry with its raw struct, without creation-time behaviour (no
// attributes, list insertion, notifications or replication).
//
export function createLocalEntityRaw(type: EntityType, data: unknown): Entity {
	const en = getFreeEntity(ENTITY_INDEX_DONT_CARE);

	ASSERT(en !== undefined, "entity heap exhausted while restoring campaign state");

	setLocalEntityType(en, type);

	setLocalEntityData(en, data);

	return en;
}
