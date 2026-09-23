//
// Isolated tests of runtime primitives that the adopted group behaviour does
// not reach by itself. Expectations come from the quoted C.
//

import { afterEach, describe, expect, it } from "vitest";
import { initialiseCampaignCore } from "../../src";
import { UnportedBehaviourError } from "../../src/core/assert";
import { getDeltaTime, setDeltaTimeFrom, setManualDeltaTime } from "../../src/core/time";
import { getUpdateSucc, setEntityUpdateFrameRate, setUpdateEntity, setUpdateSucc } from "../../src/entity/special/update/update";
import {
	deleteLocalEntityFromParentsChildList,
	getLocalEntityChildPred,
	getLocalEntityFirstChild,
	insertLocalEntityIntoParentsChildList,
	insertLocalEntityIntoParentsChildListRaw,
} from "../../src/entity/system/en_list";
import { notifyLocalEntity } from "../../src/entity/system/en_msgs";
import { createLocalEntityRaw } from "../../src/entity/system/en_heap";
import { deinitialiseEntityRuntime, type Entity } from "../../src/entity/system/entity";
import { EntityMessage, EntityType, ListType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { ScriptedClock } from "../adapters/scripted-clock";

function start(): { update: Entity; groups: Entity[] } {
	initialiseCampaignCore({ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: new RecordingEntityReplication(), clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata() });
	const update = createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {});
	setUpdateEntity(update);
	const groups: Entity[] = [];
	for (let i = 0; i < 3; i++) {
		groups.push(createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, { sleep: 0, assist_timer: 0 }));
	}
	return { update, groups };
}

afterEach(() => {
	deinitialiseEntityRuntime();
});

describe("time.c :: static initialisers", () => {
	it("starts from float system_delta_time = 0.1, converted at compile time to nearest", () => {
		start();
		// a C constant initialiser is converted when the program is compiled,
		// not by the run-time FPU, so it is 0x3dcccccd even though EECH rounds
		// its run-time arithmetic toward zero (docs/fidelity/fpu-semantics.md)
		expect(getDeltaTime()).toBe(0.10000000149011612);
	});
});

describe("time.c :: set_manual_delta_time", () => {
	it("is refused while the frame rate is locked", () => {
		start();
		const clock = new ScriptedClock();
		clock.setFrame(0.5, true);
		setDeltaTimeFrom(clock);
		setManualDeltaTime(0.25);
		expect(getDeltaTime()).toBe(0.5);
		clock.setFrame(0.5, false);
		setDeltaTimeFrom(clock);
		setManualDeltaTime(0.25);
		expect(getDeltaTime()).toBe(0.25);
	});
});

describe("up_update.c :: set_entity_update_frame_rate", () => {
	it("truncates towards zero like a C (int) cast", () => {
		start();
		const clock = new ScriptedClock();
		// time.c never measures a negative delta; the cast semantics are still C's:
		// (int) (-1.25f * 2 + 1.0) = (int) -1.5 = -1, and -1.25f / -1 = 1.25f
		clock.setFrame(-1.25, false);
		setDeltaTimeFrom(clock);
		expect(setEntityUpdateFrameRate(2)).toBe(-1);
		expect(getDeltaTime()).toBe(1.25);
		// (int) (0.3f * 2 + 1.0) = (int) 1.6000000238418579 = 1
		clock.setFrame(0.3, false);
		setDeltaTimeFrom(clock);
		expect(setEntityUpdateFrameRate(2)).toBe(1);
	});
});

describe("en_list.c :: insert / delete with notifications", () => {
	it("inserts after a predecessor and deletes from the middle", () => {
		const { update, groups } = start();
		const [a, b, c] = groups;
		insertLocalEntityIntoParentsChildList(a, ListType.LIST_TYPE_UPDATE, update, undefined);
		insertLocalEntityIntoParentsChildList(c, ListType.LIST_TYPE_UPDATE, update, a);
		insertLocalEntityIntoParentsChildList(b, ListType.LIST_TYPE_UPDATE, update, a);
		expect(getLocalEntityChildPred(c, ListType.LIST_TYPE_UPDATE)).toBe(b);
		deleteLocalEntityFromParentsChildList(b, ListType.LIST_TYPE_UPDATE);
		expect(getLocalEntityChildPred(c, ListType.LIST_TYPE_UPDATE)).toBe(a);
		deleteLocalEntityFromParentsChildList(a, ListType.LIST_TYPE_UPDATE);
		expect(getLocalEntityFirstChild(update, ListType.LIST_TYPE_UPDATE)).toBe(c);
		expect(getLocalEntityChildPred(c, ListType.LIST_TYPE_UPDATE)).toBeUndefined();
	});

	it("deleting an entity that is in no list does nothing", () => {
		const { update, groups } = start();
		deleteLocalEntityFromParentsChildList(groups[0], ListType.LIST_TYPE_UPDATE);
		expect(getLocalEntityFirstChild(update, ListType.LIST_TYPE_UPDATE)).toBeUndefined();
	});

	it("#ifdef DEBUG: inserting an entity already in the list is fatal", () => {
		const { update, groups } = start();
		insertLocalEntityIntoParentsChildList(groups[0], ListType.LIST_TYPE_UPDATE, update, undefined);
		insertLocalEntityIntoParentsChildList(groups[1], ListType.LIST_TYPE_UPDATE, update, undefined);
		expect(() => insertLocalEntityIntoParentsChildList(groups[0], ListType.LIST_TYPE_UPDATE, update, undefined)).toThrow(
			expect.objectContaining({
				name: "EechFatalError",
				format: "Entity already in list (entity type = %s, list type = %s)",
				message: "Entity already in list (entity type = ENTITY_TYPE_GROUP, list type = LIST_TYPE_UPDATE)",
			}),
		);
	});

	it("an entity whose recorded parent no longer lists it may be inserted (the C check walks the list)", () => {
		const { update, groups } = start();
		// a stale parent pointer, as en_list.c notes can happen with ENTITY_ATTR_PARENT
		groups[0].links.update_link = { parent: update, child_succ: undefined, child_pred: undefined };
		insertLocalEntityIntoParentsChildList(groups[0], ListType.LIST_TYPE_UPDATE, update, undefined);
		expect(getLocalEntityFirstChild(update, ListType.LIST_TYPE_UPDATE)).toBe(groups[0]);
	});
});

describe("up_msgs.c :: response_to_unlink_child", () => {
	it("advances update_succ when the pending successor is unlinked", () => {
		const { update, groups } = start();
		const [a, b, c] = groups;
		insertLocalEntityIntoParentsChildListRaw(a, ListType.LIST_TYPE_UPDATE, update, undefined);
		insertLocalEntityIntoParentsChildListRaw(b, ListType.LIST_TYPE_UPDATE, update, a);
		insertLocalEntityIntoParentsChildListRaw(c, ListType.LIST_TYPE_UPDATE, update, b);
		setUpdateSucc(b);
		deleteLocalEntityFromParentsChildList(b, ListType.LIST_TYPE_UPDATE);
		expect(getUpdateSucc()).toBe(c);
		deleteLocalEntityFromParentsChildList(a, ListType.LIST_TYPE_UPDATE);
		expect(getUpdateSucc()).toBe(c);
	});

	it("ignores other list types", () => {
		const { update, groups } = start();
		setUpdateSucc(groups[0]);
		expect(notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, update, groups[0], ListType.LIST_TYPE_MEMBER)).toBe(1);
		expect(getUpdateSucc()).toBe(groups[0]);
	});
});

describe("gp_msgs.c :: response_to_link_parent", () => {
	it("LIST_TYPE_DIVISION (set_local_division_name) is not ported", () => {
		const { update, groups } = start();
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_PARENT, groups[0], update, ListType.LIST_TYPE_DIVISION)).toThrow(UnportedBehaviourError);
	});
});
