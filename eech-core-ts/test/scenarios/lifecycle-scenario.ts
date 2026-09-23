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
// and, since Slice 5b (taskgen.c :: create_supply_task -> create_task, issue #14):
//
//   observe-tasks    print tasks, their routes and task lists, and the forces' supply task counters
//   single-player    a single player session: nothing is transmitted (or packed)
//   game-type        the front end's game type
//   keysite-landing  a keysite's raw landing types and usable state, as a saved game holds them
//   group-alive      a restored group's raw alive bit
//   task-counter     a force's raw task generation counter (task_generation [sub_type].created)
//   sector-state     a sector's raw side presence and surface-to-air defence levels
//
// and, since Slice 6a (assign.c :: assign_keysite_tasks, issue #16):
//
//   member-count     a group's raw member count, as a saved game holds it (gp_pack.c ::
//                    unpack_local_data; its live maintenance is not ported)
//   group-sleep      a restored group's raw sleep timer
//   air-register     a group on its force's LIST_TYPE_AIR_REGISTRY list (appended)
//   aircraft-type    a restored member's aircraft sub type
//   add-member       an aircraft member (helicopter or fixed wing) appended to a group's member list
//   unassigned-task  a task on a keysite's LIST_TYPE_UNASSIGNED_TASK list (and its objective's
//                    LIST_TYPE_TASK_DEPENDENT list unless NULL), as a saved game holds it
//   pilot            a pilot entity (pilot lock holder)
//   pilot-lock       a task or group on a pilot's LIST_TYPE_PILOT_LOCK list (appended)
//   assign-tasks     assign_keysite_tasks (keysite, category)
//
// Reaching assign_primary_task_to_group (the slice 6a boundary) ends the
// scenario with "result boundary assign_primary_task_to_group <group> <task>",
// as the C harness's boundary trap does.
//
// Tasks the original creates are labelled task<index> when first printed.
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

import { EechAssertionError, EechFatalError, EechNullDereferenceError, UnportedBoundaryError } from "../../src/core/assert";
import { assignKeysiteTasks } from "../../src/ai/taskgen/assign";
import type { AircraftRaw } from "../../src/entity/mobile/aircraft/ac_float";
import { storeUnsignedBitfield } from "../../src/core/cint";
import { toFloat32 } from "../../src/core/float32";
import { setGameStatus } from "../../src/core/game-status";
import { setGameType } from "../../src/core/game-type";
import { initialiseCampaignCore } from "../../src";
import { clearedTaskGeneration, type ForceRaw } from "../../src/entity/special/force/force";
import { updateKeysiteCargo, type KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { assessGroupSupplies, type GroupRaw } from "../../src/entity/special/group/group";
import { clearedTaskRaw, type TaskRaw } from "../../src/entity/special/task/task";
import type { WaypointRaw } from "../../src/entity/special/waypoint/waypoint";
import { setCommsModel, type CommsModel } from "../../src/entity/system/comms";
import { createLocalSectorEntities, getLocalRawSectorEntity, type SectorRaw } from "../../src/entity/special/sector/sector";
import { setEntityCommsTransmission } from "../../src/entity/system/en_comms";
import type { EntityAttribute } from "../../src/entity/system/en_attrs";
import { createClientServerEntity } from "../../src/entity/system/en_creat";
import { destroyClientServerEntityFamily } from "../../src/entity/system/en_dstry";
import { createLocalEntityRaw, getFirstFreeEntity, getFreeEntity, getLocalEntityList, getLocalEntitySucc } from "../../src/entity/system/en_heap";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { getLocalEntityIntValue, getLocalEntityVec3dPtr } from "../../src/entity/system/en_values";
import { getWorldMap, setEntityWorldMapSize } from "../../src/entity/system/en_world";
import { getLocalEntityData, setLocalEntityData, setLocalEntityType, setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { EntitySide, EntitySubTypeTask, EntityType, IntType, ListType, TaskStateType, Vec3dType, type EntityType as EntityTypeT } from "../../src/generated/c-enums";
import type { CampaignEvents, EntityReplication, ReplicatedEntityAttribute, ReplicatedTaskRoute } from "../../src/ports";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { ScriptedClock } from "../adapters/scripted-clock";
import type { KeysiteSpec, PositionSpec } from "./campaign-scenario";
import { observeSupplyTasks as observeSupplyTaskCalls, traceForceLowOnSupplies } from "./supply-boundary";
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
	| { kind: "assess-group"; group: string }
	| { kind: "observe-tasks" }
	| { kind: "single-player" }
	| { kind: "game-type"; type: number }
	| { kind: "keysite-landing"; keysite: string; landingTypes: number; usableState: number }
	| { kind: "group-alive"; group: string; alive: number }
	| { kind: "task-counter"; force: string; subType: number; created: number }
	// sector_side [BLUE], [RED]; surface_to_air_defence_level [NEUTRAL], [BLUE], [RED]
	| { kind: "sector-state"; sector: string; blue: number; red: number; samNeutral: number; samBlue: number; samRed: number }
	| { kind: "member-count"; group: string; count: number }
	| { kind: "group-sleep"; group: string; sleep: number }
	| { kind: "air-register"; group: string }
	| { kind: "aircraft-type"; member: string; subType: number }
	// objective: an entity label or "NULL"
	| { kind: "unassigned-task"; label: string; keysite: string; objective: string; subType: number; side: number; critical: number; priority: number; expire: number }
	| { kind: "add-member"; label: string; group: string; type: number; subType: number; x: number; z: number }
	| { kind: "pilot"; label: string }
	| { kind: "pilot-lock"; entity: string; pilot: string }
	| { kind: "assign-tasks"; keysite: string; category: number };

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
		private readonly taskLabelOfIndex: (index: number) => string,
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

	public transmitTaskPointers(taskIndex: number, route: ReplicatedTaskRoute): void {
		let text = `transmit-task-pointers ${this.taskLabelOfIndex(taskIndex)} nodes`;

		for (const node of route.nodes) {
			text += ` ${float32Hex(node.x)} ${float32Hex(node.y)} ${float32Hex(node.z)}`;
		}

		text += " formations";

		for (const formation of route.formationTypes) {
			text += ` ${formation}`;
		}

		text += " waypoints";

		for (const waypoint of route.waypointTypes) {
			text += ` ${waypoint}`;
		}

		text += " dependents";

		for (const dependent of route.dependentIndices) {
			text += ` ${this.labelOfIndex(dependent)}`;
		}

		this.lines.push(`${text} return ${this.labelOfIndex(route.returnKeysiteIndex)}`);
	}

	public transmitSwitchParent(entityIndex: number, type: ListType, parentIndex: number): void {
		this.lines.push(`transmit-switch-parent ${this.taskLabelOfIndex(entityIndex)} ${type} ${this.labelOfIndex(parentIndex)}`);
	}
}

// The campaign screen's MISSION_CREATED response, as the C harness records it
class LineCampaignEvents implements CampaignEvents {
	public constructor(
		private readonly lines: string[],
		private readonly taskLabelOfIndex: (index: number) => string,
	) {}

	public missionCreated(taskIndex: number): void {
		this.lines.push(`campaign mission-created ${this.taskLabelOfIndex(taskIndex)}`);
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

	// a task the original creates is labelled task<index> when it is first printed
	const taskLabelOfIndex = (index: number): string => {
		if (labels[index] === undefined) {
			labels[index] = `task${index}`;
		}

		return labelOfIndex(index);
	};

	const labelNewTasks = (): void => {
		for (let en = getLocalEntityList(); en !== undefined; en = getLocalEntitySucc(en)) {
			if (en.type === EntityType.ENTITY_TYPE_TASK) {
				taskLabelOfIndex(en.index);
			}
		}
	};

	const objects = new InMemoryObject3DMetadata();

	const physical = new InMemoryMobilePhysicalState();

	initialiseCampaignCore(
		{
			mobilePhysicalState: physical,
			entityReplication: new LineReplication(lines, labelOfIndex, taskLabelOfIndex),
			clock: new ScriptedClock(),
			object3DMetadata: objects,
			campaignEvents: new LineCampaignEvents(lines, taskLabelOfIndex),
		},
		{ numberOfEntities: spec.heap },
	);

	let observeSupplyTasks = false;

	let observeTasks = false;

	traceForceLowOnSupplies((d) => lines.push(`message ${labelOf(d.receiver)} ${labelOf(d.sender)} ${d.message} ${d.subType}`));

	// create_supply_task's calls are printed only once the scenario observes them
	observeSupplyTaskCalls(labelOf, (line) => {
		if (observeSupplyTasks) {
			lines.push(line);
		}
	});

	const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
	labels[session.index] = "session";
	setSessionEntityRaw(session);

	const update = createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {});
	labels[update.index] = "update";
	setUpdateEntity(update);

	const forces: Entity[] = [];

	for (let i = 0; i < spec.forces.length; i++) {
		const raw: ForceRaw = { side: spec.forces[i], task_generation: clearedTaskGeneration() };
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
			landing_types: 0,
			keysite_usable_state: 0,
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

				labelNewTasks();
			} else if (op.kind === "observe-supply-tasks") {
				observeSupplyTasks = true;
			} else if (op.kind === "comms-model") {
				setCommsModel(op.model as CommsModel);
			} else if (op.kind === "restore-group") {
				const raw: GroupRaw = {
					sub_type: op.subType,
					side: op.side,
					alive: 0,
					supplies: { ammo_supply_level: toFloat32(op.ammo), fuel_supply_level: toFloat32(op.fuel) },
					sleep: 0,
					assist_timer: 0,
					member_count: 0,
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
					const leaderRaw: AircraftRaw = { mob: { sub_type: 0 } };
					const leader = createLocalEntityRaw(EntityType.ENTITY_TYPE_HELICOPTER, leaderRaw);
					labels[leader.index] = `${op.label}.leader`;
					insertLocalEntityIntoParentsChildListRaw(leader, ListType.LIST_TYPE_MEMBER, group, undefined);
					physical.setMobilePosition(leader.index, { x: toFloat32(op.leader.x), y: 0, z: toFloat32(op.leader.z) });
				}
			} else if (op.kind === "task") {
				const raw: TaskRaw = clearedTaskRaw();
				raw.sub_type = op.subType;
				raw.task_state = op.state;
				raw.task_user_data = toFloat32(op.userData);
				raw.side = storeUnsignedBitfield(op.side, 2);
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

				labelNewTasks();
			} else if (op.kind === "observe-tasks") {
				observeTasks = true;
			} else if (op.kind === "single-player") {
				setEntityCommsTransmission(false);
			} else if (op.kind === "game-type") {
				setGameType(op.type);
			} else if (op.kind === "keysite-landing") {
				const raw = getLocalEntityData<KeysiteRaw>(find(op.keysite) as Entity);

				raw.landing_types = storeUnsignedBitfield(op.landingTypes, 4);
				raw.keysite_usable_state = storeUnsignedBitfield(op.usableState, 3);
			} else if (op.kind === "task-counter") {
				getLocalEntityData<ForceRaw>(find(op.force) as Entity).task_generation[op.subType].created = op.created;
			} else if (op.kind === "group-alive") {
				getLocalEntityData<GroupRaw>(find(op.group) as Entity).alive = storeUnsignedBitfield(op.alive, 1);
			} else if (op.kind === "sector-state") {
				// scenario input: narrowed to nearest, as the C harness parses it
				const raw = getLocalEntityData<SectorRaw>(find(op.sector) as Entity);

				raw.sector_side[EntitySide.ENTITY_SIDE_BLUE_FORCE] = toFloat32(op.blue);
				raw.sector_side[EntitySide.ENTITY_SIDE_RED_FORCE] = toFloat32(op.red);
				raw.surface_to_air_defence_level[EntitySide.ENTITY_SIDE_NEUTRAL] = toFloat32(op.samNeutral);
				raw.surface_to_air_defence_level[EntitySide.ENTITY_SIDE_BLUE_FORCE] = toFloat32(op.samBlue);
				raw.surface_to_air_defence_level[EntitySide.ENTITY_SIDE_RED_FORCE] = toFloat32(op.samRed);
			} else if (op.kind === "member-count") {
				// gp_pack.c :: unpack_local_data: raw->member_count (unsigned int : NUM_MEMBER_COUNT_BITS)
				getLocalEntityData<GroupRaw>(find(op.group) as Entity).member_count = storeUnsignedBitfield(op.count, 6);
			} else if (op.kind === "group-sleep") {
				getLocalEntityData<GroupRaw>(find(op.group) as Entity).sleep = toFloat32(op.sleep);
			} else if (op.kind === "air-register") {
				const group = find(op.group) as Entity;
				const side = getLocalEntityIntValue(group, IntType.INT_TYPE_SIDE);

				for (const force of forces) {
					if ((force.data as ForceRaw).side === side) {
						insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_AIR_REGISTRY, force, lastChild(force, ListType.LIST_TYPE_AIR_REGISTRY));
						break;
					}
				}
			} else if (op.kind === "aircraft-type") {
				getLocalEntityData<AircraftRaw>(find(op.member) as Entity).mob.sub_type = op.subType;
			} else if (op.kind === "unassigned-task") {
				const raw: TaskRaw = clearedTaskRaw();
				raw.sub_type = op.subType;
				raw.side = storeUnsignedBitfield(op.side, 2);
				raw.task_state = TaskStateType.TASK_STATE_UNASSIGNED;
				raw.critical_task = storeUnsignedBitfield(op.critical, 1);
				raw.task_priority = toFloat32(op.priority);
				raw.expire_timer = toFloat32(op.expire);
				const task = createLocalEntityRaw(EntityType.ENTITY_TYPE_TASK, raw);
				labels[task.index] = op.label;
				const keysite = find(op.keysite) as Entity;
				insertLocalEntityIntoParentsChildListRaw(task, ListType.LIST_TYPE_UNASSIGNED_TASK, keysite, lastChild(keysite, ListType.LIST_TYPE_UNASSIGNED_TASK));
				const objective = find(op.objective);

				if (objective !== undefined) {
					insertLocalEntityIntoParentsChildListRaw(task, ListType.LIST_TYPE_TASK_DEPENDENT, objective, lastChild(objective, ListType.LIST_TYPE_TASK_DEPENDENT));
				}
			} else if (op.kind === "add-member") {
				if (op.type !== EntityType.ENTITY_TYPE_HELICOPTER && op.type !== EntityType.ENTITY_TYPE_FIXED_WING) {
					throw new Error("add-member: not an aircraft entity type");
				}

				const raw: AircraftRaw = { mob: { sub_type: op.subType } };
				const member = createLocalEntityRaw(op.type, raw);
				labels[member.index] = op.label;
				const group = find(op.group) as Entity;
				insertLocalEntityIntoParentsChildListRaw(member, ListType.LIST_TYPE_MEMBER, group, lastChild(group, ListType.LIST_TYPE_MEMBER));
				physical.setMobilePosition(member.index, { x: toFloat32(op.x), y: 0, z: toFloat32(op.z) });
			} else if (op.kind === "pilot") {
				const pilot = createLocalEntityRaw(EntityType.ENTITY_TYPE_PILOT, {});
				labels[pilot.index] = op.label;
			} else if (op.kind === "pilot-lock") {
				const pilot = find(op.pilot) as Entity;
				insertLocalEntityIntoParentsChildListRaw(find(op.entity) as Entity, ListType.LIST_TYPE_PILOT_LOCK, pilot, lastChild(pilot, ListType.LIST_TYPE_PILOT_LOCK));
			} else if (op.kind === "assign-tasks") {
				assignKeysiteTasks(find(op.keysite), op.category);
			} else {
				destroyClientServerEntityFamily(find(op.label) as Entity);
			}

		}
	} catch (e) {

		if (e instanceof UnportedBoundaryError) {
			// the slice 6a boundary: assign_primary_task_to_group (group_en, task_en)
			result = `boundary assign_primary_task_to_group ${labelOf(e.args[0] as Entity)} ${taskLabelOfIndex((e.args[1] as Entity).index)}`;
		} else if (e instanceof EechAssertionError) {
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

	// tasks, their routes and lists, and the forces' task counters, once observed
	if (observeTasks) {
		for (let en = getLocalEntityList(); en !== undefined; en = getLocalEntitySucc(en)) {
			if (en.type === EntityType.ENTITY_TYPE_FORCE) {
				lines.push(`force ${labelOf(en)} supply-tasks-created ${getLocalEntityData<ForceRaw>(en).task_generation[EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY].created}`);
			} else if (en.type === EntityType.ENTITY_TYPE_TASK) {
				const raw = getLocalEntityData<TaskRaw>(en);

				lines.push(
					`task ${labelOf(en)} ${en.index} sub ${raw.sub_type} side ${raw.side} state ${raw.task_state} id ${raw.task_id} critical ${raw.critical_task} ` +
						`movement ${raw.movement_type} length ${raw.route_length} difficulty ${raw.difficulty} expire ${float32Hex(raw.expire_timer)} ` +
						`priority ${float32Hex(raw.task_priority)} user ${float32Hex(raw.task_user_data)} ` +
						`objective ${labelOf(getLocalEntityParent(en, ListType.LIST_TYPE_TASK_DEPENDENT))} keysite ${labelOf(getLocalEntityParent(en, ListType.LIST_TYPE_UNASSIGNED_TASK))} ` +
						`sector ${labelOf(getLocalEntityParent(en, ListType.LIST_TYPE_SECTOR_TASK))} update ${labelOf(getLocalEntityParent(en, ListType.LIST_TYPE_UPDATE))}`,
				);

				const nodes = raw.route_nodes;

				if (nodes !== undefined) {
					const waypoints = raw.route_waypoint_types as number[];
					const formations = raw.route_formation_types as number[];
					const dependents = raw.route_dependents as (Entity | undefined)[];

					let text = `route ${labelOf(en)}`;

					for (let i = 0; i <= raw.route_length; i++) {
						text += ` ${float32Hex(nodes[i].x)} ${float32Hex(nodes[i].y)} ${float32Hex(nodes[i].z)} ${waypoints[i]} ${formations[i]} ${labelOf(dependents[i])}`;
					}

					lines.push(`${text} return ${labelOf(raw.return_keysite)}`);
				}
			}
		}

		for (const keysite of keysites) {
			lines.push(`unassigned ${labelOf(keysite)}${listText(keysite, ListType.LIST_TYPE_UNASSIGNED_TASK)}`);
			lines.push(`dependents ${labelOf(keysite)}${listText(keysite, ListType.LIST_TYPE_TASK_DEPENDENT)}`);
		}

		if (mapComplete) {
			const map = getWorldMap();

			for (let z = map.min_map_z_sector; z <= map.max_map_z_sector; z++) {
				for (let x = map.min_map_x_sector; x <= map.max_map_x_sector; x++) {
					const sector = getLocalRawSectorEntity(x, z) as Entity;

					if (getLocalEntityFirstChild(sector, ListType.LIST_TYPE_SECTOR_TASK) !== undefined) {
						lines.push(`sector-tasks ${labelOf(sector)}${listText(sector, ListType.LIST_TYPE_SECTOR_TASK)}`);
					}
				}
			}
		}
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
		} else if (op.kind === "observe-tasks") {
			lines.push("observe-tasks");
		} else if (op.kind === "single-player") {
			lines.push("single-player");
		} else if (op.kind === "game-type") {
			lines.push(`game-type ${op.type}`);
		} else if (op.kind === "keysite-landing") {
			lines.push(`keysite-landing ${op.keysite} ${op.landingTypes} ${op.usableState}`);
		} else if (op.kind === "task-counter") {
			lines.push(`task-counter ${op.force} ${op.subType} ${op.created}`);
		} else if (op.kind === "group-alive") {
			lines.push(`group-alive ${op.group} ${op.alive}`);
		} else if (op.kind === "sector-state") {
			lines.push(`sector-state ${op.sector} ${[op.blue, op.red, op.samNeutral, op.samBlue, op.samRed].map((n) => formatNumber(n)).join(" ")}`);
		} else if (op.kind === "member-count") {
			lines.push(`member-count ${op.group} ${op.count}`);
		} else if (op.kind === "group-sleep") {
			lines.push(`group-sleep ${op.group} ${formatNumber(op.sleep)}`);
		} else if (op.kind === "air-register") {
			lines.push(`air-register ${op.group}`);
		} else if (op.kind === "aircraft-type") {
			lines.push(`aircraft-type ${op.member} ${op.subType}`);
		} else if (op.kind === "unassigned-task") {
			lines.push(`unassigned-task ${op.label} ${op.keysite} ${op.objective} ${op.subType} ${op.side} ${op.critical} ${formatNumber(op.priority)} ${formatNumber(op.expire)}`);
		} else if (op.kind === "add-member") {
			lines.push(`add-member ${op.label} ${op.group} ${op.type} ${op.subType} ${formatNumber(op.x)} ${formatNumber(op.z)}`);
		} else if (op.kind === "pilot") {
			lines.push(`pilot ${op.label}`);
		} else if (op.kind === "pilot-lock") {
			lines.push(`pilot-lock ${op.entity} ${op.pilot}`);
		} else if (op.kind === "assign-tasks") {
			lines.push(`assign-tasks ${op.keysite} ${op.category}`);
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
