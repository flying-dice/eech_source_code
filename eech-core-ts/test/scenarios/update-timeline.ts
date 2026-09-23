//
// Update timeline scenarios (slice 2), shared by every conformance runner:
// vitest, the Lua 5.1 runner and the C reference harness (serialiseTimeline).
//
// A timeline restores groups (raw timers and update-list membership, as a
// saved campaign holds them), then runs steps:
//
//   set:   set_client_server_entity_float_value (group, FLOAT_TYPE_SLEEP |
//          FLOAT_TYPE_ASSIST_TIMER, value): the real server setter
//   frame: the host flight loop (flight.c): the Clock supplies the frame's
//          delta and locked flag (set_delta_time), then the real
//          update_client_server_entities () runs `count` times
//          (time acceleration; 0 = paused)
//
// The frame driver is the environment. It decides nothing about what time
// does; the ported update loop and group update function do.
//
// TSTL-compatible: no Node APIs, no Map/Set, no JSON, no undefined properties.
//

import { EechAssertionError } from "../../src/core/assert";
import { toFloat32 } from "../../src/core/float32";
import { getDeltaTime } from "../../src/core/time";
import { initialiseCampaignCore, setDeltaTime } from "../../src";
import type { GroupRaw } from "../../src/entity/special/group/group";
import { setUpdateEntity, updateClientServerEntities } from "../../src/entity/special/update/update";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { getLocalEntityFloatValue, setClientServerEntityFloatValue } from "../../src/entity/system/en_values";
import { createLocalEntityRaw } from "../../src/entity/system/en_heap";
import type { Entity } from "../../src/entity/system/entity";
import { EntitySide, EntitySubTypeGroup, EntityType, FloatType, ListType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { ScriptedClock } from "../adapters/scripted-clock";

export interface TimelineGroupSpec {
	subType: EntitySubTypeGroup;
	side: EntitySide;
	sleep: number;
	assist: number;
	// restored on LIST_TYPE_UPDATE (in spec order)
	onUpdateList: boolean;
}

export type TimelineStep =
	| { kind: "set"; group: number; floatType: FloatType; value: number }
	| { kind: "frame"; delta: number; locked: boolean; count: number };

export interface TimelineSpec {
	entityUpdateFrameRate: number;
	groups: TimelineGroupSpec[];
	steps: TimelineStep[];
}

export interface TimelineTransmit {
	step: number;
	entity: string;
	floatType: number;
	value: number;
}

export interface TimelineGroupState {
	sleep: number;
	assist: number;
}

export interface TimelineStepState {
	delta: number;
	// labels of LIST_TYPE_UPDATE in list order
	updateList: string[];
	groups: TimelineGroupState[];
}

export interface TimelineOutcome {
	// "ok" or "assert:<C expression>"
	result: string;
	transmissions: TimelineTransmit[];
	// one entry per completed step
	steps: TimelineStepState[];
}

// Runaway guard (test protection, not campaign policy): a frame may not ask
// for more update passes than this; see validateTimeline.
export const MAX_UPDATE_PASSES_PER_FRAME = 10000;

export function validateTimeline(spec: TimelineSpec): void {
	for (const step of spec.steps) {
		if (step.kind === "frame") {
			const passes = step.count * (step.delta * spec.entityUpdateFrameRate + 1);
			if (!(step.delta > 0) || step.count < 0 || passes > MAX_UPDATE_PASSES_PER_FRAME) {
				throw new Error(`timeline frame outside the test bounds (delta ${step.delta}, count ${step.count})`);
			}
		}
	}
}

export function runTimeline(spec: TimelineSpec): TimelineOutcome {
	validateTimeline(spec);

	const replication = new RecordingEntityReplication();
	const clock = new ScriptedClock();

	initialiseCampaignCore(
		{ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: replication, clock, object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: new RecordingCampaignEvents() },
		{ entityUpdateFrameRate: spec.entityUpdateFrameRate },
	);

	const labels: Record<number, string> = {};

	const update = createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {});

	setUpdateEntity(update);

	const groups: Entity[] = [];

	let tail: Entity | undefined = undefined;

	for (let i = 0; i < spec.groups.length; i++) {
		const g = spec.groups[i];
		const raw: GroupRaw = {
			sub_type: g.subType,
			side: g.side,
			alive: 0,
			supplies: { ammo_supply_level: 0, fuel_supply_level: 0 },
			sleep: toFloat32(g.sleep),
			assist_timer: toFloat32(g.assist),
			member_count: 0,
			group_list_type: 0,
		};
		const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, raw);
		labels[group.index] = `group${i}`;
		if (g.onUpdateList) {
			insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_UPDATE, update, tail);
			tail = group;
		}
		groups.push(group);
	}

	const outcome: TimelineOutcome = { result: "ok", transmissions: [], steps: [] };

	const collectTransmissions = (step: number): void => {
		for (const t of replication.transmissions.splice(0, replication.transmissions.length)) {
			outcome.transmissions.push({ step, entity: labels[t.entityIndex], floatType: t.type, value: t.value });
		}
	};

	for (let s = 0; s < spec.steps.length; s++) {
		const step = spec.steps[s];

		try {
			if (step.kind === "set") {
				// scenario input: narrowed to nearest, as the C harness parses it (next_float)
				setClientServerEntityFloatValue(groups[step.group], step.floatType, toFloat32(step.value));
			} else {
				clock.setFrame(toFloat32(step.delta), step.locked);

				setDeltaTime();

				for (let c = 0; c < step.count; c++) {
					updateClientServerEntities();
				}
			}
		} catch (e) {
			if (e instanceof EechAssertionError) {
				collectTransmissions(s);
				outcome.result = `assert:${e.expression}`;
				return outcome;
			}
			throw e;
		}

		collectTransmissions(s);

		const updateList: string[] = [];
		let en = getLocalEntityFirstChild(update, ListType.LIST_TYPE_UPDATE);
		while (en) {
			updateList.push(labels[en.index]);
			en = getLocalEntityChildSucc(en, ListType.LIST_TYPE_UPDATE);
		}

		const states: TimelineGroupState[] = [];
		for (const group of groups) {
			states.push({
				sleep: getLocalEntityFloatValue(group, FloatType.FLOAT_TYPE_SLEEP),
				assist: getLocalEntityFloatValue(group, FloatType.FLOAT_TYPE_ASSIST_TIMER),
			});
		}

		outcome.steps.push({ delta: getDeltaTime(), updateList, groups: states });
	}

	return outcome;
}

// Input for c-reference/harness.c (timeline mode).
export function serialiseTimeline(spec: TimelineSpec, formatNumber: (n: number) => string): string {
	const lines: string[] = [`timeline ${spec.entityUpdateFrameRate}`];

	for (const g of spec.groups) {
		lines.push(`tgroup ${g.subType} ${g.side} ${formatNumber(g.sleep)} ${formatNumber(g.assist)} ${g.onUpdateList ? 1 : 0}`);
	}

	for (const step of spec.steps) {
		if (step.kind === "set") {
			lines.push(`set ${step.group} ${step.floatType} ${formatNumber(step.value)}`);
		} else {
			lines.push(`frame ${formatNumber(step.delta)} ${step.locked ? 1 : 0} ${step.count}`);
		}
	}

	lines.push("end");

	return lines.join("\n") + "\n";
}
