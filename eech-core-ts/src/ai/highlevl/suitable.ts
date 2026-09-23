//
// How well each kind of group suits each kind of task.
//
// C provenance: ai/highlevl/suitable.c :: calculate_group_to_task_suitability,
//               get_group_to_task_suitability, initialise_group_task_array
//
// highlevl.c :: initialise_highlevel_ai builds the table once at start-up
// from the group and task databases (generated from gp_dbase.c and
// ts_dbase.c); initialiseCampaignCore does the same. deinitialise_group_task_array
// is the next initialisation's reset.
//

import { ASSERT, assertNotNullDereference } from "../../core/assert";
import { cBit, cBitAnd } from "../../core/cint";
import { f32Div, f32Mul, toFloat32RTZ } from "../../core/float32";
import { min } from "../../core/maths/miscmath";
import { EntitySubTypeGroup, EntitySubTypeTask, MovementType } from "../../generated/c-enums";
import {
	GROUP_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH,
	GROUP_DATABASE_AI_STATS_CARGO_SPACE,
	GROUP_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH,
	GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED,
	GROUP_DATABASE_AI_STATS_MOVEMENT_STEALTH,
	GROUP_DATABASE_AI_STATS_TROOP_SPACE,
	GROUP_DATABASE_DEFAULT_ENGAGE_ENEMY,
	GROUP_DATABASE_DEFAULT_LANDING_TYPE,
	GROUP_DATABASE_MOVEMENT_TYPE,
} from "../../generated/c-group-database";
import {
	TASK_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH,
	TASK_DATABASE_AI_STATS_CARGO_SPACE,
	TASK_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH,
	TASK_DATABASE_AI_STATS_MOVEMENT_SPEED,
	TASK_DATABASE_AI_STATS_MOVEMENT_STEALTH,
	TASK_DATABASE_AI_STATS_TROOP_SPACE,
	TASK_DATABASE_ENGAGE_ENEMY,
	TASK_DATABASE_LANDING_TYPES,
	TASK_DATABASE_MOVEMENT_TYPE,
} from "../../generated/c-task-database";

// C provenance: suitable.c :: static float **group_task_array
let group_task_array: number[][] | undefined = undefined;

// C provenance: suitable.c :: calculate_group_to_task_suitability (its
// ASSERT (group_task_array) holds while initialise_group_task_array runs it)
export function calculateGroupToTaskSuitability(group_type: EntitySubTypeGroup, task_type: EntitySubTypeTask): number {
	//
	// Critical factors
	//

	// Movement type

	if (TASK_DATABASE_MOVEMENT_TYPE[task_type] !== MovementType.MOVEMENT_TYPE_ALL && TASK_DATABASE_MOVEMENT_TYPE[task_type] !== GROUP_DATABASE_MOVEMENT_TYPE[group_type]) {
		return 0.0;
	}

	const landing_type = GROUP_DATABASE_DEFAULT_LANDING_TYPE[group_type];

	if (cBitAnd(TASK_DATABASE_LANDING_TYPES[task_type], cBit(landing_type)) === 0) {
		return 0.0;
	}

	// Movement speed

	if (TASK_DATABASE_AI_STATS_MOVEMENT_SPEED[task_type] > GROUP_DATABASE_AI_STATS_MOVEMENT_SPEED[group_type]) {
		return 0.0;
	}

	// Movement stealth
	//
	// Unreachable with EECH's databases: only BDA and RECON need stealth (5),
	// and every group that has passed the checks above for them has it.
	// test/unit/supply-task-construction.test.ts recomputes this from the
	// generated databases; docs/port-manifest.md lists the exclusion.

	/* istanbul ignore if */
	if (TASK_DATABASE_AI_STATS_MOVEMENT_STEALTH[task_type] > GROUP_DATABASE_AI_STATS_MOVEMENT_STEALTH[group_type]) {
		return 0.0;
	}

	// Cargo space

	if (TASK_DATABASE_AI_STATS_CARGO_SPACE[task_type] > GROUP_DATABASE_AI_STATS_CARGO_SPACE[group_type]) {
		return 0.0;
	}

	// Troop space

	if (TASK_DATABASE_AI_STATS_TROOP_SPACE[task_type] > GROUP_DATABASE_AI_STATS_TROOP_SPACE[group_type]) {
		return 0.0;
	}

	// Ground Attack Strength

	if (TASK_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH[task_type] > GROUP_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH[group_type]) {
		return 0.0;
	}

	// Air Attack Strength

	if (TASK_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH[task_type] > GROUP_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH[group_type]) {
		return 0.0;
	}

	// Engage enemy (If task requires engagement - group must be capable)

	if (TASK_DATABASE_ENGAGE_ENEMY[task_type] !== 0) {
		if (GROUP_DATABASE_DEFAULT_ENGAGE_ENEMY[group_type] === 0) {
			return 0.0;
		}
	}

	//
	// Non Critical factors: result *= min ((a / b), 1.0f), float throughout
	//

	let result = 1.0;

	if (TASK_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH[task_type] > 0) {
		const a = toFloat32RTZ(GROUP_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH[group_type]);

		const b = toFloat32RTZ(TASK_DATABASE_AI_STATS_AIR_ATTACK_STRENGTH[task_type]);

		result = f32Mul(result, min(f32Div(a, b), 1.0));
	}

	if (TASK_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH[task_type] > 0) {
		const a = toFloat32RTZ(GROUP_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH[group_type]);

		const b = toFloat32RTZ(TASK_DATABASE_AI_STATS_GROUND_ATTACK_STRENGTH[task_type]);

		result = f32Mul(result, min(f32Div(a, b), 1.0));
	}

	ASSERT(result >= 0.0 && result <= 1.0, "(result >= 0.0) && (result <= 1.0)");

	return result;
}

// C provenance: suitable.c :: get_group_to_task_suitability
export function getGroupToTaskSuitability(group_type: EntitySubTypeGroup, task_type: EntitySubTypeTask): number {
	// C indexes group_task_array without a check; initialiseCampaignCore builds it
	assertNotNullDereference(group_task_array, "group_task_array [group_type][task_type]");

	return group_task_array[group_type][task_type];
}

// C provenance: suitable.c :: initialise_group_task_array (after deinitialise_group_task_array)
export function initialiseGroupTaskArray(): void {
	const table: number[][] = [];

	for (let group = 0; group < EntitySubTypeGroup.NUM_ENTITY_SUB_TYPE_GROUPS; group++) {
		const row: number[] = [];

		for (let task = 0; task < EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS; task++) {
			row.push(calculateGroupToTaskSuitability(group, task));
		}

		table.push(row);
	}

	group_task_array = table;
}
