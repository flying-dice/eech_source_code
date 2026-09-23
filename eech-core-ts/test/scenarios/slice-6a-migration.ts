//
// Migration of the Slice 6a scenario corpus to Slice 6b's state invariants
// (issue #18): a test-oracle correction, not a campaign behaviour change.
//
// Slice 6a's corpus stopped at assign_primary_task_to_group, the decision
// boundary, so it restored only what the decision reads. Its tasks had no
// route (route_length 0, route_nodes NULL), most scenarios had no world map,
// and generated aircraft could lie outside the map. None of that is a state
// EECH can hold, and Slice 6b's transaction reads all of it: the original C
// then reads an uninitialised pointer (croute.c:445) or recurses without end.
//
// The migration is mechanical and only adds or corrects state:
//
//   1. every legacy `unassigned-task` becomes a `persisted-task` with a route
//      and a NULL return keysite (an unassigned task's, as ts_pack.c restores
//      it). A SUPPLY task gets create_supply_task's shape (taskgen.c): PICK_UP
//      at its keysite (dependent: the keysite), PREPARE_FOR_DROP_OFF 4 km
//      before the drop-off, DROP_OFF at the requester (its objective when that
//      is a keysite, else the first other keysite; dependent: that keysite),
//      FINISH_DROP_OFF 2 km beyond, the intermediate points bounded to the
//      adjusted map area and rounded to whole metres. Any other task type gets
//      a two-node NAVIGATION route from its keysite to that requester: it
//      stops at the 6a decision boundary and never reads it.
//
//   2. only for a scenario whose Slice 6a result selected a SUPPLY task (the
//      scenarios that now run the transaction):
//      - a world map when there was none (10 x 4 sectors of 8192 m), created
//        just before the first assign-tasks operation, so that no entity index
//        before it changes;
//      - a flat terrain (0 m) and a road table of one linked node at the map's
//        centre, at the same place;
//      - aircraft positions (group leaders, added members) clamped into the map
//        area;
//      - a heap of at least 400 entities when a map is added (its sectors).
//
// Everything else is unchanged. The migration report
// (docs/slices/supply-task-assignment-fixture-migration.md) and its test
// (test/c-reference/slice-6a-migration.cref.test.ts) check each migrated
// scenario against the Slice 6a baseline output of the original C.
//
// TSTL-compatible: the hand matrix (supply-task-assignment.cases.ts) uses it
// and runs under Lua 5.1.
//

import { EntitySubTypeTask, EntitySubTypeWaypoint, FormationType } from "../../src/generated/c-enums";
import type { LifecycleOp, LifecycleSpec, PersistedRouteNode } from "./lifecycle-scenario";

// Slice 6a's route-less task restoration (removed from the runners in 6b)
export interface LegacyUnassignedTask {
	kind: "unassigned-task";
	label: string;
	keysite: string;
	objective: string;
	subType: number;
	side: number;
	critical: number;
	priority: number;
	expire: number;
}

export type LegacyOp = LifecycleOp | LegacyUnassignedTask;

export interface LegacySpec {
	heap: number;
	forces: LifecycleSpec["forces"];
	keysites: LifecycleSpec["keysites"];
	ops: LegacyOp[];
}

export interface MigratedSpec {
	spec: LifecycleSpec;
	// the corrections applied, in order (the migration report lists them)
	corrections: string[];
}

// the map added to a transaction scenario without one
export const MIGRATION_MAP_X_SECTORS = 10;
export const MIGRATION_MAP_Z_SECTORS = 4;
export const MIGRATION_MAP_SIDE_LENGTH = 8192;

const MAP_PERIMETER_SIZE = 5000;

function clampInto(value: number, lower: number, upper: number): number {
	return value < lower ? lower : value > upper ? upper : value;
}

function keysiteIndex(label: string): number {
	return label.substring(0, 7) === "keysite" ? Number(label.substring(7)) : -1;
}

// the persisted route of a legacy task (rule 1)
export function persistedRoute(spec: LegacySpec, task: LegacyUnassignedTask, maxX: number, maxZ: number): PersistedRouteNode[] {
	const own = keysiteIndex(task.keysite);

	let requester = keysiteIndex(task.objective);

	if (requester < 0) {
		requester = spec.keysites.length > 1 ? (own === 0 ? 1 : 0) : own;
	}

	const start = spec.keysites[own];

	const stop = spec.keysites[requester];

	const requesterLabel = `keysite${requester}`;

	if (task.subType !== EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY) {
		return [
			{ x: start.x, y: 0, z: start.z, waypointType: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, formation: FormationType.FORMATION_ROW_LEFT, dependent: "NULL" },
			{ x: stop.x, y: 0, z: stop.z, waypointType: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_NAVIGATION, formation: FormationType.FORMATION_ROW_LEFT, dependent: "NULL" },
		];
	}

	const dx = stop.x - start.x;
	const dz = stop.z - start.z;
	const length = Math.sqrt(dx * dx + dz * dz);
	const ux = length > 0 ? dx / length : 0;
	const uz = length > 0 ? dz / length : 0;

	const adjusted = (value: number, max: number): number => Math.round(clampInto(value, MAP_PERIMETER_SIZE, max - MAP_PERIMETER_SIZE));

	return [
		{ x: start.x, y: 0, z: start.z, waypointType: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PICK_UP, formation: FormationType.FORMATION_ROW_LEFT, dependent: task.keysite },
		{
			x: adjusted(stop.x - ux * 4000, maxX),
			y: 0,
			z: adjusted(stop.z - uz * 4000, maxZ),
			waypointType: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_PREPARE_FOR_DROP_OFF,
			formation: FormationType.FORMATION_ROW_LEFT,
			dependent: "NULL",
		},
		{ x: stop.x, y: 0, z: stop.z, waypointType: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_DROP_OFF, formation: FormationType.FORMATION_ROW_LEFT, dependent: requesterLabel },
		{
			x: adjusted(stop.x + ux * 2000, maxX),
			y: 0,
			z: adjusted(stop.z + uz * 2000, maxZ),
			waypointType: EntitySubTypeWaypoint.ENTITY_SUB_TYPE_WAYPOINT_FINISH_DROP_OFF,
			formation: FormationType.FORMATION_ROW_LEFT,
			dependent: "NULL",
		},
	];
}

//
// Migrates one Slice 6a scenario. `transaction`: its Slice 6a result selected
// a SUPPLY task, so it now runs the Slice 6b transaction (rule 2).
//
export function migrateSlice6aSpec(legacy: LegacySpec, transaction: boolean): MigratedSpec {
	const corrections: string[] = [];

	let mapOp: { xSectors: number; zSectors: number; sideLength: number } | undefined = undefined;

	for (const op of legacy.ops) {
		if (op.kind === "map") {
			mapOp = { xSectors: op.xSectors, zSectors: op.zSectors, sideLength: op.sideLength };
		}
	}

	const addMap = transaction && mapOp === undefined;

	const map = mapOp === undefined ? { xSectors: MIGRATION_MAP_X_SECTORS, zSectors: MIGRATION_MAP_Z_SECTORS, sideLength: MIGRATION_MAP_SIDE_LENGTH } : mapOp;

	const maxX = map.xSectors * map.sideLength - 1;
	const maxZ = map.zSectors * map.sideLength - 1;

	const ops: LifecycleOp[] = [];

	let environmentAdded = false;

	let routes = 0;

	let clamped = 0;

	for (const op of legacy.ops) {
		// the world map and its environment exist from campaign load, while the
		// comms model is still the server's: before any comms model switch, and
		// before the first assignment
		if ((op.kind === "assign-tasks" || op.kind === "comms-model") && transaction && !environmentAdded) {
			if (addMap) {
				ops.push({ kind: "map", xSectors: map.xSectors, zSectors: map.zSectors, sideLength: map.sideLength });
			}

			ops.push({ kind: "terrain", defaultElevation: 0, cellSize: 1, cellsX: 0, cellsZ: 0, cells: [] });
			ops.push({ kind: "road-node", x: Math.floor(maxX / 2), y: 0, z: Math.floor(maxZ / 2), links: 1 });

			environmentAdded = true;
		}

		if (op.kind === "unassigned-task") {
			ops.push({
				kind: "persisted-task",
				label: op.label,
				keysite: op.keysite,
				objective: op.objective,
				subType: op.subType,
				side: op.side,
				critical: op.critical,
				priority: op.priority,
				expire: op.expire,
				route: persistedRoute(legacy, op, maxX, maxZ),
				returnKeysite: "NULL",
			});

			routes++;
		} else if (op.kind === "restore-group" && transaction && op.leader.kind === "at") {
			const x = clampInto(op.leader.x, 0, maxX);
			const z = clampInto(op.leader.z, 0, maxZ);

			if (x !== op.leader.x || z !== op.leader.z) {
				clamped++;
			}

			ops.push({ ...op, leader: { kind: "at", x, z } });
		} else if (op.kind === "add-member" && transaction) {
			const x = clampInto(op.x, 0, maxX);
			const z = clampInto(op.z, 0, maxZ);

			if (x !== op.x || z !== op.z) {
				clamped++;
			}

			ops.push({ ...op, x, z });
		} else {
			ops.push(op);
		}
	}

	if (routes > 0) {
		corrections.push(`${routes} task route(s) restored`);
	}

	if (addMap) {
		corrections.push(`world map ${map.xSectors}x${map.zSectors}@${map.sideLength} added`);
	}

	if (environmentAdded) {
		corrections.push("terrain and road network added");
	}

	if (clamped > 0) {
		corrections.push(`${clamped} aircraft position(s) clamped into the map`);
	}

	const heap = addMap && legacy.heap < 400 ? 400 : legacy.heap;

	if (heap !== legacy.heap) {
		corrections.push(`heap ${legacy.heap} -> ${heap}`);
	}

	return { spec: { heap, forces: legacy.forces, keysites: legacy.keysites, ops }, corrections };
}
