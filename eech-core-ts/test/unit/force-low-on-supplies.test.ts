//
// Slice 5a behaviour matrix (test/scenarios/force-low-on-supplies.cases.ts)
// under JavaScript semantics. The same cases run against the original C in
// test/c-reference/force-low-on-supplies.cref.test.ts and under Lua 5.1.
//

import { describe, expect, it } from "vitest";
import { initialiseCampaignCore } from "../../src";
import { EechUndefinedBehaviourError } from "../../src/core/assert";
import { setGameStatus } from "../../src/core/game-status";
import { createLocalEntityRaw } from "../../src/entity/system/en_heap";
import { insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { notifyLocalEntity } from "../../src/entity/system/en_msgs";
import { setSessionEntityRaw } from "../../src/entity/system/entity";
import { EntityMessage, EntitySide, EntitySubTypeCargo, EntitySubTypeKeysite, EntityType, GameStatusType, ListType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { ScriptedClock } from "../adapters/scripted-clock";
import { FORCE_LOW_ON_SUPPLIES_CASES } from "../scenarios/force-low-on-supplies.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";

describe("fc_msgs.c :: response_to_force_low_on_supplies", () => {
	for (const c of FORCE_LOW_ON_SUPPLIES_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			const output = runLifecycle(c.spec);

			expect(firstUnmatchedLine(output, c.expected), output.join("\n")).toBe("");

			for (const prefix of c.absent) {
				expect(output.filter((line) => line.startsWith(prefix)), `absent: ${prefix}`).toEqual([]);
			}
		});
	}

	it("refuses a cargo sub type its switch has no case for (C reads factory uninitialised)", () => {
		initialiseCampaignCore(
			{ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: new RecordingEntityReplication(), clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: new RecordingCampaignEvents() },
		);
		const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
		setSessionEntityRaw(session);
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, { side: EntitySide.ENTITY_SIDE_BLUE_FORCE });
		insertLocalEntityIntoParentsChildListRaw(force, ListType.LIST_TYPE_FORCE, session, undefined);
		const keysite = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, {
			sub_type: EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP,
			side: EntitySide.ENTITY_SIDE_BLUE_FORCE,
			alive: 1,
			in_use: 1,
			position: { x: 0, y: 0, z: 0 },
			supplies: { ammo_supply_level: 0, fuel_supply_level: 0 },
		});
		setGameStatus(GameStatusType.GAME_STATUS_INITIALISED);

		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_FORCE_LOW_ON_SUPPLIES, force, keysite, EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_SUPPLIES)).toThrow(
			EechUndefinedBehaviourError,
		);
	});
});
