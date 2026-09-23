//
// Entity runtime state.
//
// C provenance: entity/system/en_main/en_heap.c, en_heap.h (the entity array,
//               get_local_entity_index, get_local_entity_ptr),
//               en_funcs/en_funcs.h (get_local_entity_type, get_local_entity_data)
//
// EECH keeps a global array of `entity` records. Each record carries its type,
// a pointer to a type-specific raw struct, and the `succ` / `pred` links of
// the heap's free and used lists. List roots and links live inside the raw
// struct. The port keeps the same shape: `data` is the raw struct and `roots` /
// `links` hold the list slots named after the C struct members (e.g.
// `group_link`, `member_root`). Setting new raw data starts with empty slots,
// as a freshly allocated and cleared raw struct does.
//
// Records are materialised on first use, in the state initialise_entity_heap
// leaves them in (en_heap.ts), so EECH's 125000-entity heap costs nothing
// until entities are allocated. `succ` / `pred` are indices, -1 for NULL.
//
// createLocalEntityRaw () establishes an entity the way loading a saved
// campaign does (en_pack.c unpacks raw fields and list pointers directly),
// without running creation-time behaviour. Creation proper is en_creat.ts.
//

import { ASSERT } from "../../core/assert";
import { CommsModelType, EntityType, type EntityMessage } from "../../generated/c-enums";
import type { CampaignPorts } from "../../ports";
import { setCommsModel } from "./comms";

export interface EntityListRoot {
	first_child: Entity | undefined;
}

export interface EntityListLink {
	parent: Entity | undefined;
	child_succ: Entity | undefined;
	child_pred: Entity | undefined;
}

export interface Entity {
	readonly index: number;
	type: EntityType;
	data: unknown;
	roots: Record<string, EntityListRoot>;
	links: Record<string, EntityListLink>;
	// en_heap.c free / used list links, as entity indices (-1: NULL)
	succ: number;
	pred: number;
}

export type UnportedMessagePolicy = "throw" | "record";

export type MessageArg = number | Entity | undefined;

export interface UnportedMessageDelivery {
	message: EntityMessage;
	receiver: Entity;
	sender: Entity | undefined;
	args: MessageArg[];
	provenance: string;
}

interface EntityRuntime {
	ports: CampaignPorts;
	// en_heap.c :: number_of_entities, entities, first_free_entity, first_used_entity
	numberOfEntities: number;
	entities: Record<number, Entity>;
	firstFreeEntity: number;
	firstUsedEntity: number;
	unportedMessagePolicy: UnportedMessagePolicy;
	unportedMessageLog: UnportedMessageDelivery[];
	sessionEntity: Entity | undefined;
}

let runtime: EntityRuntime | undefined;

export function initialiseEntityRuntime(ports: CampaignPorts, unportedMessagePolicy: UnportedMessagePolicy): void {
	runtime = {
		ports,
		numberOfEntities: 0,
		entities: {},
		firstFreeEntity: -1,
		firstUsedEntity: -1,
		unportedMessagePolicy,
		unportedMessageLog: [],
		sessionEntity: undefined,
	};

	setCommsModel(CommsModelType.COMMS_MODEL_SERVER);
}

export function deinitialiseEntityRuntime(): void {
	runtime = undefined;
}

function getRuntime(): EntityRuntime {
	ASSERT(runtime !== undefined, "entity runtime initialised");
	return runtime;
}

export function getCampaignPorts(): CampaignPorts {
	return getRuntime().ports;
}

export function getUnportedMessagePolicy(): UnportedMessagePolicy {
	return getRuntime().unportedMessagePolicy;
}

export function recordUnportedMessage(delivery: UnportedMessageDelivery): void {
	getRuntime().unportedMessageLog.push(delivery);
}

export function takeUnportedMessageLog(): UnportedMessageDelivery[] {
	const log = getRuntime().unportedMessageLog;

	return log.splice(0, log.length);
}

// C provenance: entity/special/session/session.h :: #define get_session_entity() (session_entity)
export function getSessionEntity(): Entity | undefined {
	return getRuntime().sessionEntity;
}

export function setSessionEntityRaw(en: Entity | undefined): void {
	getRuntime().sessionEntity = en;
}

// The heap's state, for en_heap.ts.
export interface EntityHeapState {
	numberOfEntities: number;
	firstFreeEntity: number;
	firstUsedEntity: number;
}

export function getEntityHeapState(): EntityHeapState {
	return getRuntime();
}

// Replaces every record by the state initialise_entity_heap threads them into.
export function resetEntityRecords(numberOfEntities: number): void {
	const state = getRuntime();

	state.numberOfEntities = numberOfEntities;
	state.entities = {};
}

// C provenance: en_heap.h :: #define get_local_entity_ptr(INDEX) (&entities[(INDEX)])
export function getLocalEntityPtr(index: number): Entity {
	const state = getRuntime();

	ASSERT(index >= 0 && index < state.numberOfEntities, "(index >= 0) && (index < number_of_entities)");

	let en = state.entities[index];

	if (en === undefined) {
		// en_heap.c :: reset_entity_heap: memset to 0, then entities[i].succ = &entities[i + 1], .pred = &entities[i - 1]
		en = {
			index,
			type: EntityType.ENTITY_TYPE_UNKNOWN,
			data: undefined,
			roots: {},
			links: {},
			succ: index < state.numberOfEntities - 1 ? index + 1 : -1,
			pred: index > 0 ? index - 1 : -1,
		};

		state.entities[index] = en;
	}

	return en;
}

// C provenance: en_heap.h :: get_local_entity_safe_ptr for a heap link (-1: NULL)
export function getLocalEntitySafePtr(index: number): Entity | undefined {
	return index === -1 ? undefined : getLocalEntityPtr(index);
}

// C provenance: en_funcs.h :: get_local_entity_type
export function getLocalEntityType(en: Entity): EntityType {
	return en.type;
}

// C provenance: en_funcs.h :: set_local_entity_type
export function setLocalEntityType(en: Entity, type: EntityType): void {
	en.type = type;
}

// C provenance: en_funcs.h :: get_local_entity_data
export function getLocalEntityData<T>(en: Entity): T {
	return en.data as T;
}

// C provenance: en_funcs.h :: set_local_entity_data. The raw struct owns the
// list roots and links, so new raw data comes with empty ones.
export function setLocalEntityData(en: Entity, data: unknown): void {
	en.data = data;
	en.roots = {};
	en.links = {};
}
