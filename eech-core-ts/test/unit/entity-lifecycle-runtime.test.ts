//
// Isolated tests of lifecycle primitives the cargo corpus cannot reach by
// itself, with expectations derived from the C source: the arms that fail
// loudly (not ported), C undefined behaviour, and helpers used only on those
// paths. Test-only table rows stand in for entity types that are not ported
// (a vehicle, an aircraft, a fixed entity, list links of special effects).
//

import { afterEach, describe, expect, it } from "vitest";
import { initialiseCampaignCore } from "../../src";
import { EechAssertionError, EechNullDereferenceError, EechUndefinedBehaviourError, UnportedBehaviourError } from "../../src/core/assert";
import { cIntDivide, storeUnsignedBitfield, toCUnsignedInt } from "../../src/core/cint";
import { createLocalSectorEntities, getLocalRawSectorEntity, getLocalSectorEntity, removeMobileValuesFromSector } from "../../src/entity/special/sector/sector";
import { replicatedEntityAttributes, setLocalEntityAttributes } from "../../src/entity/system/en_attrs";
import { setCommsModel } from "../../src/entity/system/comms";
import { createClientServerEntity } from "../../src/entity/system/en_creat";
import { destroyClientServerEntityFamily } from "../../src/entity/system/en_dstry";
import { createLocalEntityRaw, ENTITY_INDEX_DONT_CARE, getFirstFreeEntity, getFreeEntity, getLocalEntityList, getLocalEntitySucc, setFreeEntity } from "../../src/entity/system/en_heap";
import {
	getLocalEntityFirstChild,
	insertLocalEntityIntoParentsChildListRaw,
	overloadEntityListLink,
	unlinkLocalEntityChildren,
} from "../../src/entity/system/en_list";
import { notifyLocalEntity } from "../../src/entity/system/en_msgs";
import { fnGetLocalEntityIntValue, fnSetLocalEntityRawFloatValue } from "../../src/entity/system/en_values";
import { intBitCount, setEntityWorldMapSize } from "../../src/entity/system/en_world";
import { deinitialiseEntityRuntime, type Entity } from "../../src/entity/system/entity";
import { setUpdateEntity } from "../../src/entity/special/update/update";
import { CommsModelType, EntityMessage, EntityType, FloatType, IntType, ListType, Vec3dType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { GridTerrainElevation } from "../adapters/grid-terrain-elevation";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { InMemoryRoadNetwork } from "../adapters/in-memory-road-network";
import { RecordingCampaignEvents } from "../adapters/recording-campaign-events";
import { ScriptedClock } from "../adapters/scripted-clock";

// EECH narrows toward zero (docs/fidelity/fpu-semantics.md): 1.1 -> 0x3f8ccccc, 0.1 -> 0x3dcccccc
const F_1_1_RTZ = 1.0999999046325684;
const F_0_1_RTZ = 0.09999999403953552;

let replication: RecordingEntityReplication;

function start(withMap = true): { sector: Entity; cargo: Entity } {
	replication = new RecordingEntityReplication();
	initialiseCampaignCore({ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: replication, clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: new RecordingCampaignEvents() }, { numberOfEntities: 32 });
	setUpdateEntity(createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {}));
	if (withMap) {
		setEntityWorldMapSize(1, 1, 1024);
		createLocalSectorEntities();
	}
	const cargo = withMap ? (createClientServerEntity(EntityType.ENTITY_TYPE_CARGO, ENTITY_INDEX_DONT_CARE, []) as Entity) : (undefined as unknown as Entity);
	return { sector: withMap ? (getLocalRawSectorEntity(0, 0) as Entity) : (undefined as unknown as Entity), cargo };
}

// A test-only mobile of an unported type whose IDENTIFY_* / ALIVE values are chosen
function mobileLike(type: EntityType, values: Partial<Record<IntType, number>>): Entity {
	for (const key of [IntType.INT_TYPE_ALIVE, IntType.INT_TYPE_IDENTIFY_FIXED, IntType.INT_TYPE_IDENTIFY_AIRCRAFT, IntType.INT_TYPE_IDENTIFY_VEHICLE]) {
		fnGetLocalEntityIntValue.overload(type, key, () => values[key] ?? 0);
	}
	return createLocalEntityRaw(type, {});
}

afterEach(() => {
	setCommsModel(CommsModelType.COMMS_MODEL_SERVER);
	deinitialiseEntityRuntime();
});

describe("C integer semantics", () => {
	it("integer division by zero is undefined behaviour (SIGFPE), never a value", () => {
		expect(() => cIntDivide(1, 0)).toThrow(EechUndefinedBehaviourError);
		expect(cIntDivide(-7, 2)).toBe(-3);
	});

	it("unsigned bit-fields and unsigned int keep the low bits", () => {
		expect(storeUnsignedBitfield(-1, 2)).toBe(3);
		expect(storeUnsignedBitfield(256, 8)).toBe(0);
		expect(toCUnsignedInt(-1)).toBe(4294967295);
	});

	it("int_bit_count counts the bits of the unsigned value", () => {
		expect(intBitCount(1024)).toBe(1);
		expect(intBitCount(0)).toBe(0);
		// (unsigned int) -1024 = 0xfffffc00: 22 bits
		expect(intBitCount(-1024)).toBe(22);
	});
});

describe("world map and sectors", () => {
	it("a campaign without a world map divides by a zero sector size (EECH crashes)", () => {
		start(false);
		// world_map is zero: MID_MAP_* = 0, the position is inside the zero map, then x / 0
		expect(() => createClientServerEntity(EntityType.ENTITY_TYPE_CARGO, ENTITY_INDEX_DONT_CARE, [])).toThrow(EechUndefinedBehaviourError);
	});

	it("the sector map is read without a check before it exists", () => {
		start(false);
		setEntityWorldMapSize(1, 1, 1024);
		expect(() => getLocalSectorEntity({ x: 0, y: 0, z: 0 })).toThrow(EechNullDereferenceError);
	});

	// C reads a cell outside the created map from memory it never wrote (undefined
	// behaviour); the port finds no entity there, which the C's ASSERT (en) rejects
	it("a cell without a sector entity fails the ASSERT (en)", () => {
		start(false);
		setEntityWorldMapSize(1, 1, 1024);
		createLocalSectorEntities();
		// shrink the extents' sector count without recreating the map: x_sec 1 has no entity
		setEntityWorldMapSize(2, 1, 1024);
		expect(() => getLocalSectorEntity({ x: 1500, y: 0, z: 0 })).toThrow(new EechAssertionError("en"));
	});
});

describe("sector link responses (sc_msgs.c) arms outside the cargo lifecycle", () => {
	it("a fixed entity entering a sector needs the object dimensions (Slice 4)", () => {
		const { sector } = start();
		const fixed = mobileLike(EntityType.ENTITY_TYPE_CITY_BUILDING, { [IntType.INT_TYPE_IDENTIFY_FIXED]: 1 });
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, fixed, ListType.LIST_TYPE_SECTOR)).toThrow(UnportedBehaviourError);
	});

	it("a live vehicle entering or leaving a sector updates imap defence levels (not ported)", () => {
		const { sector } = start();
		const vehicle = mobileLike(EntityType.ENTITY_TYPE_ROUTED_VEHICLE, { [IntType.INT_TYPE_ALIVE]: 1, [IntType.INT_TYPE_IDENTIFY_VEHICLE]: 1 });
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, vehicle, ListType.LIST_TYPE_SECTOR)).toThrow(/add_mobile_values_to_sector/);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, sector, vehicle, ListType.LIST_TYPE_SECTOR)).toThrow(/remove_mobile_values_from_sector/);
	});

	it("a dead vehicle skips the imap, then side awareness is not ported", () => {
		const { sector } = start();
		const wreck = mobileLike(EntityType.ENTITY_TYPE_ROUTED_VEHICLE, { [IntType.INT_TYPE_ALIVE]: 0, [IntType.INT_TYPE_IDENTIFY_VEHICLE]: 1 });
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, wreck, ListType.LIST_TYPE_SECTOR)).toThrow(/aircraft or vehicle/);
		expect(notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, sector, wreck, ListType.LIST_TYPE_SECTOR)).toBe(1);
	});

	it("an aircraft entering a sector: fog of war and FORCE_ENTERED_SECTOR (not ported)", () => {
		const { sector } = start();
		const aircraft = mobileLike(EntityType.ENTITY_TYPE_HELICOPTER, { [IntType.INT_TYPE_ALIVE]: 1, [IntType.INT_TYPE_IDENTIFY_AIRCRAFT]: 1 });
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, aircraft, ListType.LIST_TYPE_SECTOR)).toThrow(/aircraft or vehicle/);
	});

	it("on a client the mobile values are left alone", () => {
		const { sector } = start();
		const vehicle = mobileLike(EntityType.ENTITY_TYPE_ROUTED_VEHICLE, { [IntType.INT_TYPE_ALIVE]: 1, [IntType.INT_TYPE_IDENTIFY_VEHICLE]: 1 });
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		expect(notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, sector, vehicle, ListType.LIST_TYPE_SECTOR)).toBe(1);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, vehicle, ListType.LIST_TYPE_SECTOR)).toThrow(/aircraft or vehicle/);
	});

	it("other list types are acknowledged without effect", () => {
		const { sector, cargo } = start();
		expect(notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, cargo, ListType.LIST_TYPE_SECTOR_TASK)).toBe(1);
		expect(notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, sector, cargo, ListType.LIST_TYPE_SECTOR_TASK)).toBe(1);
	});

	it("a NULL sender is dereferenced on link and asserted on unlink (ASSERT (mobile_en))", () => {
		const { sector } = start();
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_CHILD, sector, undefined, ListType.LIST_TYPE_SECTOR)).toThrow(EechNullDereferenceError);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_CHILD, sector, undefined, ListType.LIST_TYPE_SECTOR)).toThrow(new EechAssertionError("mobile_en"));
		expect(() => removeMobileValuesFromSector(sector, undefined)).toThrow(new EechAssertionError("mobile_en"));
	});
});

describe("cargo link parent responses (ac_msgs.c) arms outside the cargo lifecycle", () => {
	it("targeting and gunship targeting are not ported", () => {
		const { cargo, sector } = start();
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_PARENT, cargo, sector, ListType.LIST_TYPE_TARGET)).toThrow(UnportedBehaviourError);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_LINK_PARENT, cargo, sector, ListType.LIST_TYPE_GUNSHIP_TARGET)).toThrow(UnportedBehaviourError);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, cargo, sector, ListType.LIST_TYPE_GUNSHIP_TARGET)).toThrow(UnportedBehaviourError);
		expect(() => notifyLocalEntity(EntityMessage.ENTITY_MESSAGE_UNLINK_PARENT, cargo, sector, ListType.LIST_TYPE_UPDATE)).toThrow(UnportedBehaviourError);
	});
});

describe("destruction arms outside the cargo corpus", () => {
	// test-only: special effects are not ported, their list link stands in for sm_list.c / ss_list.c
	function specialEffect(type: EntityType, cargo: Entity): Entity {
		overloadEntityListLink(type, "special_effect_link", [ListType.LIST_TYPE_SPECIAL_EFFECT]);
		const effect = createLocalEntityRaw(type, {});
		insertLocalEntityIntoParentsChildListRaw(effect, ListType.LIST_TYPE_SPECIAL_EFFECT, cargo, undefined);
		return effect;
	}

	it("a family destroy first destroys the sound effects (not ported)", () => {
		const { cargo } = start();
		specialEffect(EntityType.ENTITY_TYPE_SOUND_EFFECT, cargo);
		expect(() => destroyClientServerEntityFamily(cargo)).toThrow(/fn_destroy_client_server_entity_family \[ENTITY_TYPE_SOUND_EFFECT\]/);
		expect(replication.events).toHaveLength(1);
	});

	it("other special effects are skipped, then unlinked by destroy_local (their unlink responses are not ported)", () => {
		const { cargo } = start();
		specialEffect(EntityType.ENTITY_TYPE_SMOKE_LIST, cargo);
		expect(() => destroyClientServerEntityFamily(cargo)).toThrow(/message_responses \[ENTITY_TYPE_CARGO\] \[ENTITY_MESSAGE_UNLINK_CHILD\]/);
		// ENTITY_COMMS_DESTROY went out before the local destroy
		expect(replication.events[replication.events.length - 1]).toEqual({ kind: "destroy", entityIndex: cargo.index });
	});

	it("unlink_local_entity_children deletes every child with its notifications", () => {
		start(false);
		const update = createLocalEntityRaw(EntityType.ENTITY_TYPE_UPDATE, {});
		const group = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {});
		const other = createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {});
		insertLocalEntityIntoParentsChildListRaw(group, ListType.LIST_TYPE_UPDATE, update, undefined);
		insertLocalEntityIntoParentsChildListRaw(other, ListType.LIST_TYPE_UPDATE, update, undefined);
		unlinkLocalEntityChildren(update, ListType.LIST_TYPE_UPDATE);
		expect(getLocalEntityFirstChild(update, ListType.LIST_TYPE_UPDATE)).toBeUndefined();
	});
});

describe("attributes and the heap outside the cargo corpus", () => {
	it("a float attribute calls the raw float setter (none is ported for cargo)", () => {
		const { cargo } = start();
		expect(() => setLocalEntityAttributes(cargo, [{ kind: "float_value", type: FloatType.FLOAT_TYPE_VELOCITY, value: 1.1 }])).toThrow(
			/fn_set_local_entity_raw_float_value \[ENTITY_TYPE_CARGO\]/,
		);
	});

	it("ENTITY_COMMS_CREATE carries floats narrowed and entities by index", () => {
		const { cargo } = start();
		expect(
			replicatedEntityAttributes([
				{ kind: "float_value", type: FloatType.FLOAT_TYPE_VELOCITY, value: 1.1 },
				{ kind: "child_pred", type: ListType.LIST_TYPE_CARGO, entity: undefined },
				{ kind: "child_pred", type: ListType.LIST_TYPE_CARGO, entity: cargo },
				{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x: 0.1, y: 0, z: 0 },
			]),
		).toEqual([
			{ kind: "float_value", type: FloatType.FLOAT_TYPE_VELOCITY, value: F_1_1_RTZ },
			{ kind: "child_pred", type: ListType.LIST_TYPE_CARGO, entityIndex: -1 },
			{ kind: "child_pred", type: ListType.LIST_TYPE_CARGO, entityIndex: cargo.index },
			{ kind: "vec3d", type: Vec3dType.VEC3D_TYPE_POSITION, x: F_0_1_RTZ, y: 0, z: 0 },
		]);
	});

	it("a float attribute is narrowed to float before the raw setter (test-only setter row)", () => {
		const { cargo } = start();
		const seen: number[] = [];
		fnSetLocalEntityRawFloatValue.overload(EntityType.ENTITY_TYPE_CARGO, FloatType.FLOAT_TYPE_VELOCITY, (_en, _type, value) => {
			seen.push(value);
		});
		setLocalEntityAttributes(cargo, [{ kind: "float_value", type: FloatType.FLOAT_TYPE_VELOCITY, value: 1.1 }]);
		expect(seen).toEqual([F_1_1_RTZ]);
	});

	it("set_free_entity into an empty free list, and of the used list's tail", () => {
		start(false);
		// update 0 (start), then 31 more fill the heap of 32
		const all: Entity[] = [];
		for (let i = 0; i < 31; i++) {
			all.push(createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {}));
		}
		expect(getFirstFreeEntity()).toBeUndefined();
		setFreeEntity(all[10]);
		expect(getFirstFreeEntity()).toBe(all[10]);
		expect(all[10].succ).toBe(-1);
		// index 0 (the update entity) is the used list's tail
		const tail = getLocalEntityList() as Entity;
		let last = tail;
		for (let en = getLocalEntitySucc(tail); en !== undefined; en = getLocalEntitySucc(en)) {
			last = en;
		}
		expect(last.index).toBe(0);
		setFreeEntity(last);
		expect(getFirstFreeEntity()).toBe(last);
		expect(last.succ).toBe(all[10].index);
		expect(getLocalEntitySucc(all[0])).toBeUndefined();
	});

	it("the map can be recreated once its sectors have been freed", () => {
		start(false);
		setEntityWorldMapSize(1, 1, 1024);
		createLocalSectorEntities();
		// sc_dstry.c :: destroy_local_sector_entities is not ported; free the entity as it would
		setFreeEntity(getLocalRawSectorEntity(0, 0) as Entity);
		setEntityWorldMapSize(2, 1, 1024);
		createLocalSectorEntities();
		expect((getLocalRawSectorEntity(1, 0) as Entity).type).toBe(EntityType.ENTITY_TYPE_SECTOR);
	});

	it("allocating by index into an empty used list (the first allocation of a restore)", () => {
		initialiseCampaignCore({ mobilePhysicalState: new InMemoryMobilePhysicalState(), entityReplication: new RecordingEntityReplication(), clock: new ScriptedClock(), object3DMetadata: new InMemoryObject3DMetadata(), terrainElevation: new GridTerrainElevation(), roadNetwork: new InMemoryRoadNetwork(), campaignEvents: new RecordingCampaignEvents() }, { numberOfEntities: 4 });
		const en = getFreeEntity(2) as Entity;
		// free list 0 1 3; used list 2
		expect(getLocalEntityList()).toBe(en);
		expect(en.succ).toBe(-1);
		expect(en.pred).toBe(-1);
		const order: number[] = [];
		for (let free = getFirstFreeEntity(); free !== undefined; free = getLocalEntitySucc(free)) {
			order.push(free.index);
		}
		expect(order).toEqual([0, 1, 3]);
	});

	it("client creation stays unported at the create tables, above the heap", () => {
		start();
		setCommsModel(CommsModelType.COMMS_MODEL_CLIENT);
		expect(() => createClientServerEntity(EntityType.ENTITY_TYPE_CARGO, 20, [])).toThrow(/fn_create_client_server_entity \[ENTITY_TYPE_CARGO\] \[COMMS_MODEL_CLIENT\]/);
		// the heap itself allocates a specific index (en_heap.c; C-reference-verified in the lifecycle matrix)
		expect(getFreeEntity(20)?.index).toBe(20);
	});

	it("restoring more entities than the heap holds is refused", () => {
		start(false);
		for (let i = 0; i < 31; i++) {
			createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {});
		}
		expect(() => createLocalEntityRaw(EntityType.ENTITY_TYPE_GROUP, {})).toThrow(EechAssertionError);
	});
});
