//
// Entity lifecycle scenarios (slice 3), shared by every conformance runner:
// vitest, the Lua 5.1 runner (test/lua/conformance.ts) and the C reference
// harness, which receives serialiseLifecycle ()'s text and runs the original
// construction and destruction path.
//
// A scenario restores a session, forces and keysites (raw, as a saved game
// holds them), then runs operations through the ported campaign functions:
//
//   map      set_entity_world_map_size + create_local_sector_entities
//   create   create_client_server_entity (type, index, attributes...)
//   destroy  destroy_client_server_entity_family
//   allocate get_free_entity with a specific index
//
// The outcome is the same text the C harness prints: transmissions, created
// indices, the result, and the entity graph (heap order, cargo values, keysite
// cargo lists, sector lists). Floats are printed as bit patterns.
//
// TSTL-compatible: no Node APIs, no Map/Set, no JSON, no undefined properties.
//

import { EechAssertionError, EechFatalError } from "../../src/core/assert";
import { toFloat32 } from "../../src/core/float32";
import { initialiseCampaignCore } from "../../src";
import type { ForceRaw } from "../../src/entity/special/force/force";
import type { KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { createLocalSectorEntities, getLocalRawSectorEntity } from "../../src/entity/special/sector/sector";
import type { EntityAttribute } from "../../src/entity/system/en_attrs";
import { createClientServerEntity } from "../../src/entity/system/en_creat";
import { destroyClientServerEntityFamily } from "../../src/entity/system/en_dstry";
import { createLocalEntityRaw, getFirstFreeEntity, getFreeEntity, getLocalEntityList, getLocalEntitySucc } from "../../src/entity/system/en_heap";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { getLocalEntityIntValue, getLocalEntityVec3dPtr } from "../../src/entity/system/en_values";
import { getWorldMap, setEntityWorldMapSize } from "../../src/entity/system/en_world";
import { setLocalEntityData, setLocalEntityType, setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { EntitySide, EntityType, IntType, ListType, Vec3dType, type EntityType as EntityTypeT } from "../../src/generated/c-enums";
import type { EntityReplication, ReplicatedEntityAttribute } from "../../src/ports";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { ScriptedClock } from "../adapters/scripted-clock";
import type { KeysiteSpec } from "./campaign-scenario";
import { float32Hex } from "./float-bits";

export type LifecycleAttribute =
	| { kind: "int"; type: number; value: number }
	| { kind: "vec3d"; type: number; x: number; y: number; z: number }
	// target: an entity label (a created entity, a keysite, any other live entity such as "sector0_0") or "NULL"
	| { kind: "parent"; type: number; target: string }
	| { kind: "pred"; type: number; target: string };

export type LifecycleOp =
	| { kind: "map"; xSectors: number; zSectors: number; sideLength: number }
	| { kind: "create"; label: string; type: number; index: number; attributes: LifecycleAttribute[] }
	| { kind: "destroy"; label: string }
	// get_free_entity (index), as restoring a saved group does; the entry gets
	// ENTITY_TYPE_GROUP and empty raw data
	| { kind: "allocate"; label: string; index: number };

export interface LifecycleSpec {
	heap: number;
	forces: EntitySide[];
	keysites: KeysiteSpec[];
	ops: LifecycleOp[];
}

// The attribute list of keysite.c :: update_keysite_cargo, for scenario building
export function crateAttributes(keysite: string, side: number, subType: number, x: number, y: number, z: number): LifecycleAttribute[] {
	return [
		{ kind: "parent", type: ListType.LIST_TYPE_CARGO, target: keysite },
		{ kind: "int", type: IntType.INT_TYPE_SIDE, value: side },
		{ kind: "int", type: IntType.INT_TYPE_ENTITY_SUB_TYPE, value: subType },
		{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x, y, z },
	];
}

class LineReplication implements EntityReplication {
	public constructor(
		private readonly lines: string[],
		private readonly labelOfIndex: (index: number) => string,
	) {}

	public transmitEntityFloatValue(entityIndex: number, type: number, value: number): void {
		this.lines.push(`transmit ${this.labelOfIndex(entityIndex)} ${type} ${float32Hex(value)}`);
	}

	public transmitEntityCreate(type: EntityTypeT, entityIndex: number, attributes: ReplicatedEntityAttribute[]): void {
		let text = `transmit-create ${type} ${entityIndex}`;

		for (const attr of attributes) {
			if (attr.kind === "int_value") {
				text += ` int ${attr.type} ${attr.value}`;
			} else if (attr.kind === "float_value") {
				text += ` float ${attr.type} ${float32Hex(attr.value)}`;
			} else if (attr.kind === "vec3d") {
				text += ` vec3d ${attr.type} ${float32Hex(attr.x)} ${float32Hex(attr.y)} ${float32Hex(attr.z)}`;
			} else {
				text += ` ${attr.kind === "parent" ? "parent" : "pred"} ${attr.type} ${this.labelOfIndex(attr.entityIndex)}`;
			}
		}

		this.lines.push(`${text} end`);
	}

	public transmitEntityDestroy(entityIndex: number): void {
		this.lines.push(`transmit-destroy ${this.labelOfIndex(entityIndex)}`);
	}
}

export function runLifecycle(spec: LifecycleSpec): string[] {
	const lines: string[] = [];

	// labels by entity index; the last entity given an index names it
	const labels: Record<number, string> = {};

	const labelOfIndex = (index: number): string => {
		if (index === -1) {
			return "NULL";
		}

		const label = labels[index];

		return label === undefined ? "" : label;
	};

	const labelOf = (en: Entity | undefined): string => (en === undefined ? "NULL" : labelOfIndex(en.index));

	initialiseCampaignCore(
		{ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: new LineReplication(lines, labelOfIndex), clock: new ScriptedClock() },
		{ unportedMessagePolicy: "throw", numberOfEntities: spec.heap },
	);

	const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
	labels[session.index] = "session";
	setSessionEntityRaw(session);

	const update = createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {});
	labels[update.index] = "update";
	setUpdateEntity(update);

	const forces: Entity[] = [];

	for (let i = 0; i < spec.forces.length; i++) {
		const raw: ForceRaw = { side: spec.forces[i] };
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, raw);
		labels[force.index] = `force${i}`;
		insertLocalEntityIntoParentsChildListRaw(force, ListType.LIST_TYPE_FORCE, session, i > 0 ? forces[i - 1] : undefined);
		forces.push(force);
	}

	const keysites: Entity[] = [];
	const keysiteTail: Record<number, Entity> = {};

	for (let i = 0; i < spec.keysites.length; i++) {
		const k = spec.keysites[i];
		const raw: KeysiteRaw = {
			sub_type: k.subType,
			side: k.side,
			in_use: k.inUse ? 1 : 0,
			position: { x: toFloat32(k.x), y: 0, z: toFloat32(k.z) },
			supplies: { ammo_supply_level: toFloat32(k.ammo), fuel_supply_level: toFloat32(k.fuel) },
		};
		const keysite = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, raw);
		labels[keysite.index] = `keysite${i}`;

		for (const force of forces) {
			if ((force.data as ForceRaw).side === k.side) {
				insertLocalEntityIntoParentsChildListRaw(keysite, ListType.LIST_TYPE_KEYSITE_FORCE, force, keysiteTail[k.side]);
				keysiteTail[k.side] = keysite;
				break;
			}
		}

		keysites.push(keysite);
	}

	// created entities by scenario label (a destroyed entity keeps its record)
	const created: Record<string, Entity> = {};

	const find = (label: string): Entity | undefined => {
		if (label === "NULL") {
			return undefined;
		}

		const en = created[label];

		if (en !== undefined) {
			return en;
		}

		for (const keysite of keysites) {
			if (labels[keysite.index] === label) {
				return keysite;
			}
		}

		for (let en = getLocalEntityList(); en !== undefined; en = getLocalEntitySucc(en)) {
			if (labels[en.index] === label) {
				return en;
			}
		}

		throw new Error(`unknown entity label ${label}`);
	};

	let mapComplete = false;

	let result = "ok";

	try {
		for (const op of spec.ops) {
			if (op.kind === "map") {
				// the graph lists sectors only after a map operation completes
				mapComplete = false;

				setEntityWorldMapSize(op.xSectors, op.zSectors, op.sideLength);

				createLocalSectorEntities();

				const map = getWorldMap();

				for (let z = map.min_map_z_sector; z <= map.max_map_z_sector; z++) {
					for (let x = map.min_map_x_sector; x <= map.max_map_x_sector; x++) {
						labels[(getLocalRawSectorEntity(x, z) as Entity).index] = `sector${x}_${z}`;
					}
				}

				mapComplete = true;
			} else if (op.kind === "create") {
				const attributes: EntityAttribute[] = [];

				for (const a of op.attributes) {
					if (a.kind === "int") {
						attributes.push({ kind: "int_value", type: a.type, value: a.value });
					} else if (a.kind === "vec3d") {
						attributes.push({ kind: "vec3d", type: a.type, x: a.x, y: a.y, z: a.z });
					} else if (a.kind === "parent") {
						attributes.push({ kind: "parent", type: a.type, entity: find(a.target) });
					} else {
						attributes.push({ kind: "child_pred", type: a.type, entity: find(a.target) });
					}
				}

				const en = createClientServerEntity(op.type, op.index, attributes) as Entity;

				labels[en.index] = op.label;
				created[op.label] = en;
				lines.push(`created ${op.label} ${en.index}`);
			} else if (op.kind === "allocate") {
				const en = getFreeEntity(op.index) as Entity;

				setLocalEntityType(en, EntityType.ENTITY_TYPE_GROUP);
				setLocalEntityData(en, {});
				labels[en.index] = op.label;
				lines.push(`allocated ${op.label} ${en.index}`);
			} else {
				destroyClientServerEntityFamily(find(op.label) as Entity);
			}
		}
	} catch (e) {
		if (e instanceof EechAssertionError) {
			result = `assert ${e.expression}`;
		} else if (e instanceof EechFatalError) {
			result = `fatal ${e.format}`;
		} else {
			throw e;
		}
	}

	lines.push(`result ${result}`);

	// the entity graph
	let text = "heap free";

	for (let en = getFirstFreeEntity(); en !== undefined; en = getLocalEntitySucc(en)) {
		text += ` ${en.index}`;
	}

	lines.push(text);

	text = "heap used";

	for (let en = getLocalEntityList(); en !== undefined; en = getLocalEntitySucc(en)) {
		text += ` ${labelOf(en)}`;
	}

	lines.push(text);

	for (let en = getLocalEntityList(); en !== undefined; en = getLocalEntitySucc(en)) {
		if (en.type === EntityType.ENTITY_TYPE_CARGO) {
			const position = getLocalEntityVec3dPtr(en, Vec3dType.VEC3D_TYPE_POSITION) as { x: number; y: number; z: number };

			lines.push(
				`cargo ${labelOf(en)} ${en.index} ${getLocalEntityIntValue(en, IntType.INT_TYPE_SIDE)} ${getLocalEntityIntValue(en, IntType.INT_TYPE_ENTITY_SUB_TYPE)} ` +
					`${getLocalEntityIntValue(en, IntType.INT_TYPE_ALIVE)} ${float32Hex(position.x)} ${float32Hex(position.y)} ${float32Hex(position.z)} ` +
					`${labelOf(getLocalEntityParent(en, ListType.LIST_TYPE_CARGO))} ${labelOf(getLocalEntityParent(en, ListType.LIST_TYPE_SECTOR))}`,
			);
		}
	}

	const listText = (parent: Entity, type: ListType): string => {
		let t = "";

		for (let child = getLocalEntityFirstChild(parent, type); child !== undefined; child = getLocalEntityChildSucc(child, type)) {
			t += ` ${labelOf(child)}`;
		}

		return t === "" ? " -" : t;
	};

	for (const keysite of keysites) {
		lines.push(`keysite ${labelOf(keysite)}${listText(keysite, ListType.LIST_TYPE_CARGO)}`);
	}

	if (mapComplete) {
		const map = getWorldMap();

		for (let z = map.min_map_z_sector; z <= map.max_map_z_sector; z++) {
			for (let x = map.min_map_x_sector; x <= map.max_map_x_sector; x++) {
				const sector = getLocalRawSectorEntity(x, z) as Entity;

				lines.push(
					`sector ${labelOf(sector)} ${sector.index} ${getLocalEntityIntValue(sector, IntType.INT_TYPE_X_SECTOR)} ${getLocalEntityIntValue(sector, IntType.INT_TYPE_Z_SECTOR)}${listText(sector, ListType.LIST_TYPE_SECTOR)}`,
				);
			}
		}
	}

	return lines;
}

// Line format read by c-reference/harness.c.
export function serialiseLifecycle(spec: LifecycleSpec, formatNumber: (n: number) => string): string {
	const lines: string[] = [`heap ${spec.heap}`, "session 1"];

	for (const side of spec.forces) {
		lines.push(`force ${side}`);
	}

	for (const k of spec.keysites) {
		lines.push(`keysite ${k.side} ${k.subType} ${k.inUse ? 1 : 0} ${formatNumber(k.x)} ${formatNumber(k.z)} ${formatNumber(k.ammo)} ${formatNumber(k.fuel)}`);
	}

	for (const op of spec.ops) {
		if (op.kind === "map") {
			lines.push(`map ${op.xSectors} ${op.zSectors} ${op.sideLength}`);
		} else if (op.kind === "create") {
			let text = `create ${op.label} ${op.type} ${op.index}`;

			for (const a of op.attributes) {
				if (a.kind === "int") {
					text += ` int ${a.type} ${a.value}`;
				} else if (a.kind === "vec3d") {
					text += ` vec3d ${a.type} ${formatNumber(a.x)} ${formatNumber(a.y)} ${formatNumber(a.z)}`;
				} else {
					text += ` ${a.kind} ${a.type} ${a.target}`;
				}
			}

			lines.push(`${text} end`);
		} else if (op.kind === "allocate") {
			lines.push(`allocate ${op.label} ${op.index}`);
		} else {
			lines.push(`destroy ${op.label}`);
		}
	}

	lines.push("end");

	return lines.join("\n") + "\n";
}

//
// Checks that the expected lines appear in the output in order. An expected
// line matches an output line equal to it, or one that continues it after a
// space. Returns the first expected line not found, or "" when all match.
//
export function firstUnmatchedLine(output: string[], expected: string[]): string {
	let position = 0;

	for (const want of expected) {
		let found = false;

		while (position < output.length) {
			const line = output[position];

			position++;

			if (line === want || line.substring(0, want.length + 1) === `${want} `) {
				found = true;
				break;
			}
		}

		if (!found) {
			return want;
		}
	}

	return "";
}
