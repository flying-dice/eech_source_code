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
// and, since Slice 4 (keysite.c :: update_keysite_cargo, issue #10):
//
//   game-status    the host's set_game_status
//   bounds         an object's entry in the 3D object database (Object3DMetadata)
//   keysite-state  a keysite's raw alive bit and height, as a saved game holds them
//   update-cargo   update_keysite_cargo (keysite, level, sub_type, size)
//
// and, since Slice 5a (fc_msgs.c :: response_to_force_low_on_supplies, issue #12):
//
//   observe-supply-tasks  print the create_supply_task boundary from here on
//   comms-model           the host's set_comms_model
//   restore-group         a group (sub type, side, supplies, parent, leader) as a saved game holds it
//   task                  a task on its objective's LIST_TYPE_TASK_DEPENDENT list, as a saved game holds it
//   waypoint              a route waypoint on its dependent's LIST_TYPE_TASK_DEPENDENT list, as a saved game holds it
//   assess-group          assess_group_supplies (group)
//
// A NULL dereference (reachable through assess_group_supplies, Slice 1) ends
// the output with "result null-dereference" and the keysites' final supply
// levels, as the C harness's fault handler writes them.
//
// Crates the original creates are labelled crate<index>. Every
// FORCE_LOW_ON_SUPPLIES delivery is printed as a message line, as the C
// harness records it, before the (now ported) force response runs; see
// supply-boundary.ts.
//
// The outcome is the same text the C harness prints: transmissions, created
// indices, the result, and the entity graph (heap order, cargo values, keysite
// cargo lists, sector lists). Floats are printed as bit patterns.
//
// TSTL-compatible: no Node APIs, no Map/Set, no JSON, no undefined properties.
//

import { EechAssertionError, EechFatalError, EechNullDereferenceError } from "../../src/core/assert";
import { storeUnsignedBitfield } from "../../src/core/cint";
import { toFloat32 } from "../../src/core/float32";
import { setGameStatus } from "../../src/core/game-status";
import { initialiseCampaignCore } from "../../src";
import type { ForceRaw } from "../../src/entity/special/force/force";
import { updateKeysiteCargo, type KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { assessGroupSupplies, type GroupRaw } from "../../src/entity/special/group/group";
import type { TaskRaw } from "../../src/entity/special/task/task";
import type { WaypointRaw } from "../../src/entity/special/waypoint/waypoint";
import { setCommsModel, type CommsModel } from "../../src/entity/system/comms";
import { createLocalSectorEntities, getLocalRawSectorEntity } from "../../src/entity/special/sector/sector";
import type { EntityAttribute } from "../../src/entity/system/en_attrs";
import { createClientServerEntity } from "../../src/entity/system/en_creat";
import { destroyClientServerEntityFamily } from "../../src/entity/system/en_dstry";
import { createLocalEntityRaw, getFirstFreeEntity, getFreeEntity, getLocalEntityList, getLocalEntitySucc } from "../../src/entity/system/en_heap";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { getLocalEntityIntValue, getLocalEntityVec3dPtr } from "../../src/entity/system/en_values";
import { getWorldMap, setEntityWorldMapSize } from "../../src/entity/system/en_world";
import { getLocalEntityData, setLocalEntityData, setLocalEntityType, setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { EntitySide, EntityType, IntType, ListType, Vec3dType, type EntityType as EntityTypeT } from "../../src/generated/c-enums";
import type { EntityReplication, ReplicatedEntityAttribute } from "../../src/ports";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { ScriptedClock } from "../adapters/scripted-clock";
import type { KeysiteSpec, PositionSpec } from "./campaign-scenario";
import { takeSupplyTaskLines, traceForceLowOnSupplies } from "./supply-boundary";
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
	| { kind: "allocate"; label: string; index: number }
	| { kind: "game-status"; status: number }
	| { kind: "bounds"; object: number; xmin: number; xmax: number; ymin: number; ymax: number; zmin: number; zmax: number }
	| { kind: "keysite-state"; keysite: string; alive: number; y: number }
	| { kind: "update-cargo"; keysite: string; level: number; subType: number; size: number }
	| { kind: "observe-supply-tasks" }
	| { kind: "comms-model"; model: number }
	// parent: a keysite label, "independent" (the force of its side) or "NULL"
	| { kind: "restore-group"; label: string; subType: number; side: number; ammo: number; fuel: number; parent: string; busy: boolean; leader: PositionSpec }
	| { kind: "task"; label: string; objective: string; subType: number; side: number; state: number; userData: number }
	| { kind: "waypoint"; label: string; dependent: string; subType: number }
	| { kind: "assess-group"; group: string };

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

	const objects = new InMemoryObject3DMetadata();

	const physical = new InMemoryMobilePhysicalState();

	// "record": the create_supply_task boundary (Slice 5b) is recorded, not thrown
	initialiseCampaignCore(
		{ mobilePhysicalState: physical, entityReplication: new LineReplication(lines, labelOfIndex), clock: new ScriptedClock(), object3DMetadata: objects },
		{ unportedMessagePolicy: "record", numberOfEntities: spec.heap },
	);

	let observeSupplyTasks = false;

	const takeBoundary = (): void => {
		for (const line of takeSupplyTaskLines(labelOf)) {
			if (observeSupplyTasks) {
				lines.push(line);
			}
		}
	};

	traceForceLowOnSupplies(
		(d) => lines.push(`message ${labelOf(d.receiver)} ${labelOf(d.sender)} ${d.message} ${d.subType}`),
		takeBoundary,
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
			alive: 0,
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

	const lastChild = (parent: Entity, type: ListType): Entity | undefined => {
		let last: Entity | undefined = undefined;

		for (let en = getLocalEntityFirstChild(parent, type); en !== undefined; en = getLocalEntityChildSucc(en, type)) {
			last = en;
		}

		return last;
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
			} else if (op.kind === "game-status") {
				setGameStatus(op.status);
			} else if (op.kind === "bounds") {
				// scenario input: narrowed to nearest, as the C harness parses it
				objects.setBoundingBox(op.object, {
					xmin: toFloat32(op.xmin),
					xmax: toFloat32(op.xmax),
					ymin: toFloat32(op.ymin),
					ymax: toFloat32(op.ymax),
					zmin: toFloat32(op.zmin),
					zmax: toFloat32(op.zmax),
				});
			} else if (op.kind === "keysite-state") {
				const raw = getLocalEntityData<KeysiteRaw>(find(op.keysite) as Entity);

				raw.alive = storeUnsignedBitfield(op.alive, 1);
				raw.position.y = toFloat32(op.y);
			} else if (op.kind === "update-cargo") {
				updateKeysiteCargo(find(op.keysite) as Entity, toFloat32(op.level), op.subType, toFloat32(op.size));

				// crates the original created are labelled by index
				for (let en = getLocalEntityList(); en !== undefined; en = getLocalEntitySucc(en)) {
					if (en.type === EntityType.ENTITY_TYPE_CARGO && labels[en.index] === undefined) {
						labels[en.index] = `crate${en.index}`;
					}
				}
			} else if (op.kind === "observe-supply-tasks") {
				observeSupplyTasks = true;
			} else if (op.kind === "comms-model") {
				setCommsModel(op.model as CommsModel);
			} else if (op.kind === "restore-group") {
				const raw: GroupRaw = {
					sub_type: op.subType,
					side: op.side,
					supplies: { ammo_supply_level: toFloat32(op.ammo), fuel_supply_level: toFloat32(op.fuel) },
					sleep: 0,
					assist_timer: 0,
				};
				const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, raw);
				labels[group.index] = op.label;

				if (op.parent === "independent") {
					for (const force of forces) {
						if ((force.data as ForceRaw).side === op.side) {
							insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_INDEPENDENT_GROUP, force, lastChild(force, ListType.LIST_TYPE_INDEPENDENT_GROUP));
							break;
						}
					}
				} else if (op.parent !== "NULL") {
					const parent = find(op.parent) as Entity;
					insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_KEYSITE_GROUP, parent, lastChild(parent, ListType.LIST_TYPE_KEYSITE_GROUP));
				}

				if (op.busy) {
					const guide = createLocalEntityRaw(EntityType.ENTITY_TYPE_GUIDE, {});
					labels[guide.index] = `${op.label}.guide`;
					insertLocalEntityIntoParentsChildListRaw(guide, ListType.LIST_TYPE_GUIDE_STACK, group, undefined);
				}

				if (op.leader.kind === "at") {
					const leader = createLocalEntityRaw(EntityType.ENTITY_TYPE_HELICOPTER, {});
					labels[leader.index] = `${op.label}.leader`;
					insertLocalEntityIntoParentsChildListRaw(leader, ListType.LIST_TYPE_MEMBER, group, undefined);
					physical.setMobilePosition(leader.index, { x: toFloat32(op.leader.x), y: 0, z: toFloat32(op.leader.z) });
				}
			} else if (op.kind === "task") {
				const raw: TaskRaw = {
					sub_type: op.subType,
					task_state: op.state,
					task_user_data: toFloat32(op.userData),
					side: storeUnsignedBitfield(op.side, 2),
				};
				const task = createLocalEntityRaw(EntityType.ENTITY_TYPE_TASK, raw);
				labels[task.index] = op.label;
				const objective = find(op.objective) as Entity;
				insertLocalEntityIntoParentsChildListRaw(task, ListType.LIST_TYPE_TASK_DEPENDENT, objective, lastChild(objective, ListType.LIST_TYPE_TASK_DEPENDENT));
			} else if (op.kind === "waypoint") {
				const raw: WaypointRaw = { sub_type: op.subType };
				const waypoint = createLocalEntityRaw(EntityType.ENTITY_TYPE_WAYPOINT, raw);
				labels[waypoint.index] = op.label;
				const dependent = find(op.dependent) as Entity;
				insertLocalEntityIntoParentsChildListRaw(waypoint, ListType.LIST_TYPE_TASK_DEPENDENT, dependent, lastChild(dependent, ListType.LIST_TYPE_TASK_DEPENDENT));
			} else if (op.kind === "assess-group") {
				assessGroupSupplies(find(op.group) as Entity);
			} else {
				destroyClientServerEntityFamily(find(op.label) as Entity);
			}

			takeBoundary();
		}
	} catch (e) {
		takeBoundary();

		if (e instanceof EechAssertionError) {
			result = `assert ${e.expression}`;
		} else if (e instanceof EechFatalError) {
			result = `fatal ${e.format}`;
		} else if (e instanceof EechNullDereferenceError) {
			// the C harness reports a NULL dereference from its fault handler,
			// which can only write the keysites' supply levels, not the graph
			lines.push("result null-dereference");

			for (const keysite of keysites) {
				const raw = keysite.data as KeysiteRaw;

				lines.push(`final keysite ${float32Hex(raw.supplies.ammo_supply_level)} ${float32Hex(raw.supplies.fuel_supply_level)}`);
			}

			return lines;
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
		} else if (op.kind === "game-status") {
			lines.push(`game-status ${op.status}`);
		} else if (op.kind === "bounds") {
			lines.push(`bounds ${op.object} ${[op.xmin, op.xmax, op.ymin, op.ymax, op.zmin, op.zmax].map((n) => formatNumber(n)).join(" ")}`);
		} else if (op.kind === "keysite-state") {
			lines.push(`keysite-state ${op.keysite} ${op.alive} ${formatNumber(op.y)}`);
		} else if (op.kind === "update-cargo") {
			lines.push(`update-cargo ${op.keysite} ${formatNumber(op.level)} ${op.subType} ${formatNumber(op.size)}`);
		} else if (op.kind === "observe-supply-tasks") {
			lines.push("observe-supply-tasks");
		} else if (op.kind === "comms-model") {
			lines.push(`comms-model ${op.model}`);
		} else if (op.kind === "restore-group") {
			const leader = op.leader.kind === "at" ? `1 ${formatNumber(op.leader.x)} ${formatNumber(op.leader.z)}` : "0 0 0";
			lines.push(`restore-group ${op.label} ${op.subType} ${op.side} ${formatNumber(op.ammo)} ${formatNumber(op.fuel)} ${op.parent} ${op.busy ? 1 : 0} ${leader}`);
		} else if (op.kind === "task") {
			lines.push(`task ${op.label} ${op.objective} ${op.subType} ${op.side} ${op.state} ${formatNumber(op.userData)}`);
		} else if (op.kind === "waypoint") {
			lines.push(`waypoint ${op.label} ${op.dependent} ${op.subType}`);
		} else if (op.kind === "assess-group") {
			lines.push(`assess-group ${op.group}`);
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
