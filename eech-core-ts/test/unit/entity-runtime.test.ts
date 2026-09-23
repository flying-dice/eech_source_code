//
// Isolated tests of the entity runtime primitives (lists, function tables,
// messages, comms dispatch) and of the loud failure modes the port adds for
// behaviour that is not ported yet.
//

import { afterEach, describe, expect, it } from "vitest";
import { initialiseCampaignCore } from "../../src";
import { EechAssertionError, UnportedBehaviourError } from "../../src/core/assert";
import { setCommsModel } from "../../src/entity/system/comms";
import { getLocalEntityChildSucc, getLocalEntityFirstChild, getLocalEntityParent, insertLocalEntityIntoParentsChildListRaw } from "../../src/entity/system/en_list";
import { notifyLocalEntity } from "../../src/entity/system/en_msgs";
import { getLocalEntityFloatValue, getLocalEntityIntValue, setClientServerEntityFloatValue } from "../../src/entity/system/en_values";
import { createLocalEntityRaw } from "../../src/entity/system/en_heap";
import { deinitialiseEntityRuntime, getCampaignPorts } from "../../src/entity/system/entity";
import { CommsModelType, EntityMessage, EntitySide, EntityType, FloatType, IntType, ListType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { ScriptedClock } from "../adapters/scripted-clock";

function ports() {
	return { mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: new RecordingEntityReplication(), clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), campaignEvents: new RecordingCampaignEvents() };
}

afterEach(() => {
	deinitialiseEntityRuntime();
});

describe("entity runtime", () => {
	it("refuses to run before initialisation", () => {
		deinitialiseEntityRuntime();
		expect(() => getCampaignPorts()).toThrow(EechAssertionError);
	});

	it("inserts at the head of a non-empty list, keeping both links consistent", () => {
		initialiseCampaignCore(ports());
		const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
		const a = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, { side: EntitySide.ENTITY_SIDE_BLUE_FORCE });
		const b = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, { side: EntitySide.ENTITY_SIDE_RED_FORCE });
		const c = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, { side: EntitySide.ENTITY_SIDE_NEUTRAL });

		insertLocalEntityIntoParentsChildListRaw(a, ListType.LIST_TYPE_FORCE, session, undefined);
		insertLocalEntityIntoParentsChildListRaw(b, ListType.LIST_TYPE_FORCE, session, undefined);
		insertLocalEntityIntoParentsChildListRaw(c, ListType.LIST_TYPE_FORCE, session, b);

		expect(getLocalEntityFirstChild(session, ListType.LIST_TYPE_FORCE)).toBe(b);
		expect(getLocalEntityChildSucc(b, ListType.LIST_TYPE_FORCE)).toBe(c);
		expect(getLocalEntityChildSucc(c, ListType.LIST_TYPE_FORCE)).toBe(a);
		expect(getLocalEntityChildSucc(a, ListType.LIST_TYPE_FORCE)).toBeUndefined();
		expect(a.links.force_link.child_pred).toBe(c);
		expect(c.links.force_link.child_pred).toBe(b);
		expect(getLocalEntityParent(c, ListType.LIST_TYPE_FORCE)).toBe(session);
	});

	it("shares one group_link between BUILDING, INDEPENDENT and KEYSITE group lists (en_list/get_prnt.h)", () => {
		initialiseCampaignCore(ports());
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, { side: EntitySide.ENTITY_SIDE_BLUE_FORCE });
		const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {});

		insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_INDEPENDENT_GROUP, force, undefined);

		expect(getLocalEntityParent(group, ListType.LIST_TYPE_KEYSITE_GROUP)).toBe(force);
		expect(getLocalEntityParent(group, ListType.LIST_TYPE_BUILDING_GROUP)).toBe(force);

		const keysite = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, {});
		expect(() => insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_KEYSITE_GROUP, keysite, undefined)).toThrow(
			new EechAssertionError("entity not already in a list sharing this link"),
		);
	});

	it("asserts pred != parent", () => {
		initialiseCampaignCore(ports());
		const session = createLocalEntityRaw(EntityType.ENTITY_TYPE_SESSION, {});
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, {});
		expect(() => insertLocalEntityIntoParentsChildListRaw(force, ListType.LIST_TYPE_FORCE, session, session)).toThrow(new EechAssertionError("pred != parent"));
	});

	it("fails loudly on list roots and links that are not ported", () => {
		initialiseCampaignCore(ports());
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, {});
		expect(() => getLocalEntityFirstChild(force, ListType.LIST_TYPE_PILOT)).toThrow(UnportedBehaviourError);
		expect(() => getLocalEntityParent(force, ListType.LIST_TYPE_UPDATE)).toThrow(new UnportedBehaviourError("list link [ENTITY_TYPE_FORCE] [LIST_TYPE_UPDATE]"));
	});

	it("fails loudly on value overloads that are not ported", () => {
		initialiseCampaignCore(ports());
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, { side: EntitySide.ENTITY_SIDE_BLUE_FORCE });
		const bridge = createLocalEntityRaw(EntityType.ENTITY_TYPE_BRIDGE, {});
		expect(() => getLocalEntityIntValue(force, IntType.INT_TYPE_KILLS)).toThrow(
			new UnportedBehaviourError("fn_get_local_entity_int_value [ENTITY_TYPE_FORCE] [INT_TYPE_KILLS]"),
		);
		expect(() => getLocalEntityFloatValue(bridge, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL)).toThrow(UnportedBehaviourError);
	});

	it("has no ported client comms model overloads", () => {
		initialiseCampaignCore(ports());
		const keysite = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, { supplies: { ammo_supply_level: 0, fuel_supply_level: 0 } });
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		expect(() => setClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, 1)).toThrow(
			new UnportedBehaviourError("fn_set_client_server_entity_float_value [COMMS_MODEL_CLIENT] [ENTITY_TYPE_KEYSITE] [FLOAT_TYPE_AMMO_SUPPLY_LEVEL]"),
		);
	});

	it("resets to the server comms model on initialisation", () => {
		initialiseCampaignCore(ports());
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		const replication = new RecordingEntityReplication();
		initialiseCampaignCore({ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: replication, clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), campaignEvents: new RecordingCampaignEvents() });
		const keysite = createLocalEntityRaw(EntityType.ENTITY_TYPE_KEYSITE, { supplies: { ammo_supply_level: 0, fuel_supply_level: 0 } });
		setClientServerEntityFloatValue(keysite, FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, 0.1);
		// the C prototype's float parameter narrows the value, toward zero (0x3dcccccc)
		expect(replication.transmissions).toEqual([{ entityIndex: keysite.index, type: FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, value: 0.09999999403953552 }]);
	});
});

describe("entity messages", () => {
	it("fails loudly on message responses not declared at all", () => {
		initialiseCampaignCore(ports());
		const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {});
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, group, undefined)).toThrow(UnportedBehaviourError);
	});

	it("asserts the message range (notify_local_entity)", () => {
		initialiseCampaignCore(ports());
		const force = createLocalEntityRaw(EntityType.ENTITY_TYPE_FORCE, {});
		expect(() => notifyLocalEntity(EntityMessage.NUM_ENTITY_MESSAGES, force, undefined)).toThrow(
			new EechAssertionError("(message >= 0) && (message < NUM_ENTITY_MESSAGES)"),
		);
		expect(() => notifyLocalEntity(-1 as EntityMessage, force, undefined)).toThrow(EechAssertionError);
	});
});
