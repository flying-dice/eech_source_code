//
// Entity runtime state.
//
// C provenance: entity/system/en_main/en_main.c (entity array, get_local_entity_index,
//               get_local_entity_type, get_local_entity_data)
//
// EECH keeps a global array of `entity` records. Each record carries its type
// and a pointer to a type-specific raw struct; list roots and links live inside
// that raw struct. The port keeps the same shape: `data` is the raw struct and
// `roots` / `links` hold the list slots named after the C struct members
// (e.g. `group_link`, `member_root`).
//
// Entity creation (en_creat.c, create_local_entity with attribute lists) is not
// ported. createLocalEntityRaw () establishes an entity the way loading a saved
// campaign does (en_pack.c unpacks raw fields and list pointers directly),
// without running creation-time behaviour.
//

import { ASSERT } from "../../core/assert";
import { CommsModelType, type EntityMessage, type EntityType } from "../../generated/c-enums";
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
	readonly type: EntityType;
	readonly data: unknown;
	readonly roots: Record<string, EntityListRoot>;
	readonly links: Record<string, EntityListLink>;
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
	entities: Entity[];
	unportedMessagePolicy: UnportedMessagePolicy;
	unportedMessageLog: UnportedMessageDelivery[];
	sessionEntity: Entity | undefined;
}

let runtime: EntityRuntime | undefined;

export function initialiseEntityRuntime(ports: CampaignPorts, unportedMessagePolicy: UnportedMessagePolicy): void {
	runtime = {
		ports,
		entities: [],
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

export function createLocalEntityRaw(type: EntityType, data: unknown): Entity {
	const state = getRuntime();

	const en: Entity = {
		index: state.entities.length,
		type,
		data,
		roots: {},
		links: {},
	};

	state.entities.push(en);

	return en;
}

// C provenance: en_main.c :: get_local_entity_type
export function getLocalEntityType(en: Entity): EntityType {
	return en.type;
}

// C provenance: en_main.c :: get_local_entity_data
export function getLocalEntityData<T>(en: Entity): T {
	return en.data as T;
}
