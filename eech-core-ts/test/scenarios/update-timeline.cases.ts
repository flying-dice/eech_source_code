//
// Behaviour matrix for slice 2: gp_updt.c :: update_server, the gp_float.c
// timer setters, up_update.c :: update_client_server_entities /
// set_entity_update_frame_rate, and time.c :: set_manual_delta_time.
//
// Expected outcomes are derived by reading the original C (the `c` field names
// the branch). Float values follow C float arithmetic, e.g. a 1.0 s frame at
// rate 2 is (int) (1.0f * 2 + 1.0) = 3 passes of 1.0f / 3. They are checked
// against JavaScript, Lua 5.1 and the executed original C.
//
// TSTL-compatible: no Node APIs.
//

import { EntitySide, EntitySubTypeGroup, FloatType } from "../../src/generated/c-enums";
import type { TimelineGroupSpec, TimelineOutcome, TimelineSpec, TimelineStep, TimelineStepState } from "./update-timeline";

export interface TimelineCase {
	id: string;
	c: string;
	spec: TimelineSpec;
	expected: TimelineOutcome;
}

const SLEEP = FloatType.FLOAT_TYPE_SLEEP;
const ASSIST = FloatType.FLOAT_TYPE_ASSIST_TIMER;

// C float values used below
const F_0_1 = 0.10000000149011612; // 0.1f: time.c's initial system_delta_time
const F_0_7 = 0.699999988079071; // 0.7f

function group(sleep: number, assist: number, onUpdateList: boolean): TimelineGroupSpec {
	return { subType: EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER, side: EntitySide.ENTITY_SIDE_BLUE_FORCE, sleep, assist, onUpdateList };
}

function set(g: number, floatType: FloatType, value: number): TimelineStep {
	return { kind: "set", group: g, floatType, value };
}

function frame(delta: number, count = 1, locked = false): TimelineStep {
	return { kind: "frame", delta, locked, count };
}

// timers: [sleep, assist] per group
function state(delta: number, updateList: string[], timers: [number, number][]): TimelineStepState {
	const groups = [];
	for (const [sleep, assist] of timers) {
		groups.push({ sleep, assist });
	}
	return { delta, updateList, groups };
}

function kase(id: string, c: string, spec: TimelineSpec, expected: Partial<TimelineOutcome>): TimelineCase {
	return { id, c, spec, expected: { result: "ok", transmissions: [], steps: [], ...expected } };
}

function timeline(rate: number, groups: TimelineGroupSpec[], steps: TimelineStep[]): TimelineSpec {
	return { entityUpdateFrameRate: rate, groups, steps };
}

export const UPDATE_TIMELINE_CASES: TimelineCase[] = [
	//
	// gp_float.c :: set_local_float_value (FLOAT_TYPE_SLEEP / FLOAT_TYPE_ASSIST_TIMER) via set_server_float_value
	//
	kase(
		"sleep-set-inserts-and-transmits",
		"value != 0.0 && !update_link.parent: insert into LIST_TYPE_UPDATE; server setter transmits",
		timeline(2, [group(0, 0, false)], [set(0, SLEEP, 1)]),
		{
			transmissions: [{ step: 0, entity: "group0", floatType: SLEEP, value: 1 }],
			steps: [state(F_0_1, ["group0"], [[1, 0]])],
		},
	),
	kase(
		"assist-set-inserts-at-head",
		"insert_local_entity_into_parents_child_list (.., get_update_entity (), NULL): pred NULL is the head",
		timeline(2, [group(1, 0, true), group(0, 0, false)], [set(1, ASSIST, 0.5)]),
		{
			transmissions: [{ step: 0, entity: "group1", floatType: ASSIST, value: 0.5 }],
			steps: [state(F_0_1, ["group1", "group0"], [[1, 0], [0, 0.5]])],
		},
	),
	kase(
		"set-zero-does-not-insert",
		"value == 0.0: no insertion (the value is still set and transmitted)",
		timeline(2, [group(0, 0, false)], [set(0, SLEEP, 0)]),
		{
			transmissions: [{ step: 0, entity: "group0", floatType: SLEEP, value: 0 }],
			steps: [state(F_0_1, [], [[0, 0]])],
		},
	),
	kase(
		"set-negative-inserts",
		"value != 0.0 includes negative values",
		timeline(2, [group(0, 0, false)], [set(0, SLEEP, -1)]),
		{
			transmissions: [{ step: 0, entity: "group0", floatType: SLEEP, value: -1 }],
			steps: [state(F_0_1, ["group0"], [[-1, 0]])],
		},
	),
	kase(
		"set-while-on-list-does-not-reinsert",
		"update_link.parent already set: no second insertion",
		timeline(2, [group(1, 0, true), group(1, 0, true)], [set(1, ASSIST, 2)]),
		{
			transmissions: [{ step: 0, entity: "group1", floatType: ASSIST, value: 2 }],
			steps: [state(F_0_1, ["group0", "group1"], [[1, 0], [1, 2]])],
		},
	),
	kase(
		"set-narrows-to-float",
		"the float parameter narrows 0.1 to 0.1f",
		timeline(2, [group(0, 0, false)], [set(0, ASSIST, 0.1)]),
		{
			transmissions: [{ step: 0, entity: "group0", floatType: ASSIST, value: F_0_1 }],
			steps: [state(F_0_1, ["group0"], [[0, F_0_1]])],
		},
	),

	//
	// gp_updt.c :: update_server
	//
	kase(
		"sleep-counts-down",
		"sleep > 0.0: sleep -= get_delta_time ()",
		timeline(2, [group(1, 0, true)], [frame(0.25)]),
		{ steps: [state(0.25, ["group0"], [[0.75, 0]])] },
	),
	kase(
		"expiry-exactly-at-zero-leaves-update-list",
		"sleep reaches 0.0 exactly; both timers 0.0: delete_local_entity_from_parents_child_list (LIST_TYPE_UPDATE)",
		timeline(1, [group(0.5, 0, true)], [frame(0.5)]),
		{ steps: [state(0.5, [], [[0, 0]])] },
	),
	kase(
		"overshoot-clamps-to-zero",
		"max (raw->sleep, 0.0f) clamps a negative result",
		timeline(1, [group(0.2, 0, true)], [frame(0.5)]),
		{ steps: [state(0.5, [], [[0, 0]])] },
	),
	kase(
		"both-timers-must-expire",
		"sleep expired but assist_timer > 0.0: stays on the update list",
		timeline(1, [group(0.2, 1, true)], [frame(0.5), frame(0.5)]),
		{ steps: [state(0.5, ["group0"], [[0, 0.5]]), state(0.5, [], [[0, 0]])] },
	),
	kase(
		"negative-timer-is-never-decremented-and-never-expires",
		"sleep < 0.0 is not > 0.0 and not == 0.0: the group stays on the update list",
		timeline(1, [group(-1, 0, true)], [frame(1), frame(1)]),
		{ steps: [state(1, ["group0"], [[-1, 0]]), state(1, ["group0"], [[-1, 0]])] },
	),
	kase(
		"zero-timers-on-list-leave-on-first-update",
		"a restored group on the list with both timers 0.0 is removed by its first update",
		timeline(2, [group(0, 0, true)], [frame(0.1)]),
		{ steps: [state(F_0_1, [], [[0, 0]])] },
	),
	kase(
		"self-removal-does-not-break-the-walk",
		"update_succ is saved before update_client_server_entity (en), so the successor is still updated",
		timeline(1, [group(0.1, 0, true), group(1, 0, true)], [frame(0.5)]),
		{ steps: [state(0.5, ["group1"], [[0, 0], [0.5, 0]])] },
	),

	//
	// up_update.c :: update_client_server_entities / set_entity_update_frame_rate, time.c :: set_manual_delta_time
	//
	kase(
		"frame-is-subdivided-into-equal-float-sub-steps",
		"iterations = (int) (1.0f * 2 + 1.0) = 3; each pass subtracts 1.0f / 3 = 0.3333333f (toward zero)",
		timeline(2, [group(1, 0, true)], [frame(1)]),
		// Rounding toward zero (docs/fidelity/fpu-semantics.md, class "timer"):
		// 1.0f / 3 truncates to 0.3333333f, and three passes leave
		// 1 - 3 * 0.3333333f = 2^-24 > 0, so the group stays on the update list
		// (to nearest, 0.33333334f overshoots, clamps to 0 and removes it).
		{ steps: [state(1, ["group0"], [[5.960464477539063e-8, 0]])] },
	),
	kase(
		"sub-step-float-rounding",
		"0.7f at rate 4: 3 passes of 0.23333333f; the frame delta is restored afterwards",
		timeline(4, [group(0, 2, true)], [frame(0.7)]),
		{ steps: [state(F_0_7, ["group0"], [[0, 1.2999999523162842]])] },
	),
	kase(
		"time-acceleration-repeats-the-update",
		"the host calls update_client_server_entities () once per time-acceleration step",
		timeline(2, [group(1, 0, true)], [frame(0.25, 3)]),
		{ steps: [state(0.25, ["group0"], [[0.25, 0]])] },
	),
	kase(
		"paused-frame-updates-nothing",
		"time acceleration 0: update_client_server_entities () is not called; the frame delta is still measured",
		timeline(2, [group(1, 0, true)], [frame(0.5, 0)]),
		{ steps: [state(0.5, ["group0"], [[1, 0]])] },
	),
	kase(
		"locked-frame-rate-is-not-subdivided",
		"locked_frame_rate: one pass with the whole delta; set_manual_delta_time is refused",
		timeline(2, [group(2, 0, true)], [frame(1, 1, true)]),
		{ steps: [state(1, ["group0"], [[1, 0]])] },
	),
	kase(
		"frame-rate-below-range-asserts",
		"set_entity_update_frame_rate: ASSERT ((frame_rate >= 1) && (frame_rate <= 100))",
		timeline(0, [group(1, 0, true)], [set(0, SLEEP, 2), frame(0.5)]),
		{
			result: "assert:(frame_rate >= 1) && (frame_rate <= 100)",
			transmissions: [{ step: 0, entity: "group0", floatType: SLEEP, value: 2 }],
			steps: [state(F_0_1, ["group0"], [[2, 0]])],
		},
	),
	kase(
		"frame-rate-above-range-asserts",
		"set_entity_update_frame_rate: ASSERT ((frame_rate >= 1) && (frame_rate <= 100))",
		timeline(101, [group(1, 0, true)], [frame(0.5)]),
		{ result: "assert:(frame_rate >= 1) && (frame_rate <= 100)" },
	),
	kase(
		"frame-rate-not-used-when-locked",
		"locked_frame_rate skips set_entity_update_frame_rate, so an out-of-range rate is not asserted",
		timeline(0, [group(1, 0, true)], [frame(0.5, 1, true)]),
		{ steps: [state(0.5, ["group0"], [[0.5, 0]])] },
	),
	kase(
		"frame-rate-not-used-when-paused",
		"no update call, no frame rate assertion",
		timeline(101, [group(1, 0, true)], [frame(0.5, 0)]),
		{ steps: [state(0.5, ["group0"], [[1, 0]])] },
	),
];
