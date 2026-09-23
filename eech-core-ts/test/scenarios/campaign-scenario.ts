//
// Campaign scenario model shared by every conformance runner:
//
//   - vitest (JavaScript semantics, coverage)
//   - test/lua/conformance.ts (the same code transpiled by TSTL, run by Lua 5.1)
//   - the C reference harness (c-reference/), which receives the same scenario
//     serialised by serialiseScenario () and runs the original EECH functions
//
// A scenario describes campaign state the way a restored saved game holds it:
// raw entity fields and list membership. Environmental state (mobile
// positions) goes to the deterministic port adapters. Nothing here decides
// campaign policy; the ported campaign code does.
//
// This file must stay TSTL-compatible: no Node APIs, no Map/Set, no JSON.
// Optional things are explicit variants ({ kind: "none" }), never `undefined`
// properties: a Lua table cannot hold nil, so `{ ...defaults, leader: undefined }`
// keeps the default leader after transpilation.
//

import { EechAssertionError, EechNullDereferenceError } from "../../src/core/assert";
import { toFloat32 } from "../../src/core/float32";
import { initialiseCampaignCore } from "../../src";
import { assessGroupSupplies, type GroupRaw } from "../../src/entity/special/group/group";
import { getClosestKeysite, type KeysiteRaw } from "../../src/entity/special/keysite/keysite";
import { clearedTaskGeneration, type ForceRaw } from "../../src/entity/special/force/force";
import { insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { createLocalEntityRaw } from "../../src/entity/system/en_heap";
import { setSessionEntityRaw, type Entity } from "../../src/entity/system/entity";
import { EntitySide, EntitySubTypeGroup, EntitySubTypeKeysite, EntityType, ListType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { ScriptedClock } from "../adapters/scripted-clock";
import { traceForceLowOnSupplies } from "./supply-boundary";

export interface KeysiteSpec {
	side: EntitySide;
	subType: EntitySubTypeKeysite;
	inUse: boolean;
	x: number;
	z: number;
	ammo: number;
	fuel: number;
}

export type PositionSpec = { kind: "none" } | { kind: "at"; x: number; z: number };

// Where the group is linked through its shared `group_link`
// (LIST_TYPE_KEYSITE_GROUP / LIST_TYPE_INDEPENDENT_GROUP / none).
export type GroupParentSpec = { kind: "none" } | { kind: "keysite"; keysite: number } | { kind: "independent" };

export interface GroupSpec {
	subType: EntitySubTypeGroup;
	side: EntitySide;
	ammo: number;
	fuel: number;
	parent: GroupParentSpec;
	// true: the group has a guide on LIST_TYPE_GUIDE_STACK (GROUP_MODE_BUSY)
	busy: boolean;
	// leader position (first LIST_TYPE_MEMBER child), or no members
	leader: PositionSpec;
}

export interface ScenarioSpec {
	session: boolean;
	// forces in LIST_TYPE_FORCE order under the session
	forces: EntitySide[];
	// keysites in LIST_TYPE_KEYSITE_FORCE order under the force of their side
	keysites: KeysiteSpec[];
	group: GroupSpec | undefined;
	op: ScenarioOp;
}

// The campaign function the scenario executes.
export type ScenarioOp =
	| { kind: "assess" } // assess_group_supplies (group)
	| {
			// get_closest_keysite (type, side, pos, min_range, want_range ? &actual_range : NULL, outside_of_range, exclude)
			kind: "closest";
			type: number;
			side: EntitySide;
			pos: PositionSpec;
			minRange: number;
			wantRange: boolean;
			outside: boolean;
			exclude: number; // keysite index or -1
	  };

export interface MessageEvent {
	receiver: string;
	sender: string;
	message: number;
	arg: number;
}

export interface TransmitEvent {
	entity: string;
	floatType: number;
	value: number;
}

export interface ScenarioOutcome {
	// "ok", "assert:<C expression>" or "null-dereference"
	result: string;
	// op closest: label of the returned keysite ("NULL" when none), "" otherwise
	closest: string;
	// op closest with wantRange: the value written to *actual_range
	closestRange?: number;
	messages: MessageEvent[];
	transmissions: TransmitEvent[];
	groupAmmo: number;
	groupFuel: number;
	keysiteAmmo: number[];
	keysiteFuel: number[];
}

function labelOf(labels: Record<number, string>, en: Entity | undefined): string {
	if (en === undefined) {
		return "NULL";
	}

	const label = labels[en.index];

	return label === undefined ? `entity${en.index}` : label;
}

export function runScenario(spec: ScenarioSpec): ScenarioOutcome {
	const physical = new InMemoryMobilePhysicalState();
	const replication = new RecordingEntityReplication();

	initialiseCampaignCore({ mobilePhysicalState: physical, entityReplication: replication, clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: new RecordingCampaignEvents() });

	const labels: Record<number, string> = {};

	// every FORCE_LOW_ON_SUPPLIES delivery, as the C harness traces it before
	// the (since Slice 5a, ported) force response runs; see supply-boundary.ts
	const messages: MessageEvent[] = [];

	// Slice 1 scenarios never set the game status, so the response returns at
	// its guard; create_supply_task is left fail-loud
	traceForceLowOnSupplies((d) => messages.push({ receiver: labelOf(labels, d.receiver), sender: labelOf(labels, d.sender), message: d.message, arg: d.subType }));

	const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});

	if (spec.session) {
		setSessionEntityRaw(session);
	}

	const forces: Entity[] = [];

	let pred: Entity | undefined = undefined;

	for (let i = 0; i < spec.forces.length; i++) {
		const raw: ForceRaw = { side: spec.forces[i], task_generation: clearedTaskGeneration() };
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, raw);
		labels[force.index] = `force${i}`;
		insertLocalEntityIntoParentsChildListRaw(force, ListType.LIST_TYPE_FORCE, session, pred);
		pred = force;
		forces.push(force);
	}

	const forceOfSide = (side: EntitySide): Entity | undefined => {
		for (const force of forces) {
			if ((force.data as ForceRaw).side === side) {
				return force;
			}
		}
		return undefined;
	};

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
		const force = forceOfSide(k.side);
		if (force !== undefined) {
			insertLocalEntityIntoParentsChildListRaw(keysite, ListType.LIST_TYPE_KEYSITE_FORCE, force, keysiteTail[force.index]);
			keysiteTail[force.index] = keysite;
		}
		keysites.push(keysite);
	}

	const g = spec.group;
	let group: Entity | undefined = undefined;
	let groupRaw: GroupRaw | undefined = undefined;

	if (g !== undefined) {
		groupRaw = {
			sub_type: g.subType,
			side: g.side,
			alive: 0,
			supplies: { ammo_supply_level: toFloat32(g.ammo), fuel_supply_level: toFloat32(g.fuel) },
			sleep: 0,
			assist_timer: 0,
			member_count: 0,
		};
		group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, groupRaw);
		labels[group.index] = "group";

		if (g.parent.kind === "keysite") {
			insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_KEYSITE_GROUP, keysites[g.parent.keysite], undefined);
		} else if (g.parent.kind === "independent") {
			const force = forceOfSide(g.side);
			if (force === undefined) {
				throw new Error("independent group needs a force of its side");
			}
			insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_INDEPENDENT_GROUP, force, undefined);
		}

		if (g.busy) {
			const guide = createLocalEntityRaw(EntityType.ENTITY_TYPE_GUIDE, {});
			insertLocalEntityIntoParentsChildListRaw(guide, ListType.LIST_TYPE_GUIDE_STACK, group, undefined);
		}

		if (g.leader.kind === "at") {
			const leader = createLocalEntityRaw(EntityType.ENTITY_TYPE_HELICOPTER, {});
			insertLocalEntityIntoParentsChildListRaw(leader, ListType.LIST_TYPE_MEMBER, group, undefined);
			physical.setMobilePosition(leader.index, { x: toFloat32(g.leader.x), y: 0, z: toFloat32(g.leader.z) });
		}
	}

	let result = "ok";
	let closest = "";
	let closestRange: number | undefined = undefined;

	try {
		const op = spec.op;

		if (op.kind === "assess") {
			if (group === undefined) {
				throw new Error("op assess needs a group");
			}

			assessGroupSupplies(group);
		} else {
			// the C float parameter narrows min_range; vec3d members are floats
			const pos = op.pos.kind === "none" ? undefined : { x: toFloat32(op.pos.x), y: 0, z: toFloat32(op.pos.z) };
			const actualRange = op.wantRange ? { value: -1 } : undefined;

			const found = getClosestKeysite(op.type, op.side, pos, toFloat32(op.minRange), actualRange, op.outside, op.exclude >= 0 ? keysites[op.exclude] : undefined);

			closest = labelOf(labels, found);
			closestRange = actualRange === undefined ? undefined : actualRange.value;
		}
	} catch (e) {
		if (e instanceof EechAssertionError) {
			result = `assert:${e.expression}`;
		} else if (e instanceof EechNullDereferenceError) {
			result = "null-dereference";
		} else {
			throw e;
		}
	}


	const transmissions: TransmitEvent[] = [];

	for (const t of replication.transmissions) {
		transmissions.push({ entity: labels[t.entityIndex], floatType: t.type, value: t.value });
	}

	const keysiteAmmo: number[] = [];
	const keysiteFuel: number[] = [];

	for (const keysite of keysites) {
		const raw = keysite.data as KeysiteRaw;
		keysiteAmmo.push(raw.supplies.ammo_supply_level);
		keysiteFuel.push(raw.supplies.fuel_supply_level);
	}

	return {
		result,
		closest,
		closestRange,
		messages,
		transmissions,
		groupAmmo: groupRaw === undefined ? 0 : groupRaw.supplies.ammo_supply_level,
		groupFuel: groupRaw === undefined ? 0 : groupRaw.supplies.fuel_supply_level,
		keysiteAmmo,
		keysiteFuel,
	};
}

//
// Line format read by c-reference/harness.c. Numbers are printed with enough
// digits to round-trip a double; the harness narrows them to float exactly as
// the TS scenario builder does.
//
export function serialiseScenario(spec: ScenarioSpec, formatNumber: (n: number) => string): string {
	const lines: string[] = [];

	lines.push(`session ${spec.session ? 1 : 0}`);

	for (const side of spec.forces) {
		lines.push(`force ${side}`);
	}

	for (const k of spec.keysites) {
		lines.push(`keysite ${k.side} ${k.subType} ${k.inUse ? 1 : 0} ${formatNumber(k.x)} ${formatNumber(k.z)} ${formatNumber(k.ammo)} ${formatNumber(k.fuel)}`);
	}

	const g = spec.group;

	if (g !== undefined) {
		const parentKind = g.parent.kind === "keysite" ? 1 : g.parent.kind === "independent" ? 2 : 0;
		const parentIndex = g.parent.kind === "keysite" ? g.parent.keysite : -1;
		const leader = g.leader.kind === "none" ? "0 0 0" : `1 ${formatNumber(g.leader.x)} ${formatNumber(g.leader.z)}`;

		lines.push(
			`group ${g.subType} ${g.side} ${formatNumber(g.ammo)} ${formatNumber(g.fuel)} ${parentKind} ${parentIndex} ${g.busy ? 1 : 0} ${leader}`,
		);
	}

	const op = spec.op;

	if (op.kind === "assess") {
		lines.push("op assess");
	} else {
		const pos = op.pos.kind === "none" ? "0 0 0" : `1 ${formatNumber(op.pos.x)} ${formatNumber(op.pos.z)}`;

		lines.push(`op closest ${op.type} ${op.side} ${pos} ${formatNumber(op.minRange)} ${op.wantRange ? 1 : 0} ${op.outside ? 1 : 0} ${op.exclude}`);
	}

	return lines.join("\n") + "\n";
}
