--
-- Smoke test of the production bundle (build/lua/eech-core.lua) from the point
-- of view of a Lua 5.1 host such as DCS World: plain function calls, Lua table
-- ports, no TypeScript runtime beyond what the bundle carries.
--

assert(_VERSION == "Lua 5.1", "expected Lua 5.1, got " .. tostring(_VERSION))

local core = dofile("build/lua/eech-core.lua")

local transmitted = {}

local missions = {}

-- Ports are objects: the core calls their functions as methods (port:method (...)),
-- so every port function receives the port table first.
core.initialiseCampaignCore({
	mobilePhysicalState = {
		getMobilePosition = function(self, entityIndex) return { x = 0, y = 0, z = 0 } end,
	},
	clock = {
		getDeltaTime = function(self) return 0.25 end,
		isFrameRateLocked = function(self) return false end,
	},
	object3DMetadata = {
		-- get_object_3d_bounding_box (object): a 2 x 1 x 3 crate for OBJECT_3D_SINGLE_CRATE
		getBoundingBox = function(self, objectIndex)
			assert(objectIndex == core.OBJECT_3D_SINGLE_CRATE, "unexpected object " .. tostring(objectIndex))
			return { xmin = -1, xmax = 1, ymin = -0.5, ymax = 0.5, zmin = -1.5, zmax = 1.5 }
		end,
	},
	entityReplication = {
		transmitEntityFloatValue = function(self, entityIndex, floatType, value)
			transmitted[#transmitted + 1] = { entityIndex = entityIndex, floatType = floatType, value = value }
		end,
		transmitEntityCreate = function(self, entityType, entityIndex, attributes)
			transmitted[#transmitted + 1] = { create = entityIndex, entityType = entityType, attributes = attributes }
		end,
		transmitEntityDestroy = function(self, entityIndex)
			transmitted[#transmitted + 1] = { destroy = entityIndex }
		end,
		transmitTaskPointers = function(self, taskIndex, route)
			transmitted[#transmitted + 1] = { taskPointers = taskIndex, route = route }
		end,
		transmitSwitchParent = function(self, entityIndex, listType, parentIndex)
			transmitted[#transmitted + 1] = { switchParent = entityIndex, listType = listType, parentIndex = parentIndex }
		end,
		-- slice 6b: the assignment transaction's messages
		transmitEntityIntValue = function(self, entityIndex, intType, value)
			transmitted[#transmitted + 1] = { entityIndex = entityIndex, intType = intType, value = value }
		end,
		transmitCreateWaypointRoute = function(self, taskIndex, route)
			transmitted[#transmitted + 1] = { waypointRoute = taskIndex, route = route }
		end,
		transmitSwitchList = function(self, entityIndex, fromType, parentIndex, toType)
			transmitted[#transmitted + 1] = { switchList = entityIndex, fromType = fromType, parentIndex = parentIndex, toType = toType }
		end,
		transmitSetGuideCriteria = function(self, guideIndex, criteriaType, valid, value)
			transmitted[#transmitted + 1] = { guideCriteria = guideIndex, criteriaType = criteriaType, valid = valid, value = value }
		end,
	},
	-- slice 6b: the map's terrain (flat) and road network (none here)
	terrainElevation = {
		getTerrainElevation = function(self, x, z) return 0 end,
	},
	roadNetwork = {
		hasRoadNodeTable = function(self) return false end,
		getTotalNumberOfRoadNodes = function(self) return 0 end,
		getRoadNodePosition = function(self, node) error("no road nodes") end,
		getRoadNodeNumberOfLinks = function(self, node) error("no road nodes") end,
	},
	campaignEvents = {
		missionCreated = function(self, taskIndex)
			missions[#missions + 1] = taskIndex
		end,
		missionAssigned = function(self, taskIndex)
			missions[#missions + 1] = taskIndex
		end,
	},
}, { numberOfEntities = 64 })

local T, L = core.EntityType, core.ListType

local session = core.createLocalEntityRaw(T.ENTITY_TYPE_SESSION, {})
core.setSessionEntityRaw(session)

local task_generation = {}
for i = 1, core.EntitySubTypeTask.NUM_ENTITY_SUB_TYPE_TASKS do
	task_generation[i] = { created = 0 }
end
local force = core.createLocalEntityRaw(T.ENTITY_TYPE_FORCE, { side = core.EntitySide.ENTITY_SIDE_BLUE_FORCE, task_generation = task_generation })
core.insertLocalEntityIntoParentsChildListRaw(force, L.LIST_TYPE_FORCE, session, nil)

local keysite = core.createLocalEntityRaw(T.ENTITY_TYPE_KEYSITE, {
	sub_type = core.EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP,
	side = core.EntitySide.ENTITY_SIDE_BLUE_FORCE,
	alive = 1,
	in_use = 1,
	position = { x = 0, y = 0, z = 0 },
	supplies = { ammo_supply_level = 50, fuel_supply_level = 50 },
	landing_types = 0,
	keysite_usable_state = 0,
})
core.insertLocalEntityIntoParentsChildListRaw(keysite, L.LIST_TYPE_KEYSITE_FORCE, force, nil)

local group_raw = {
	sub_type = core.EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER,
	side = core.EntitySide.ENTITY_SIDE_BLUE_FORCE,
	alive = 1,
	supplies = { ammo_supply_level = 30, fuel_supply_level = 40 },
	sleep = 0,
	assist_timer = 0,
}
local group = core.createLocalEntityRaw(T.ENTITY_TYPE_GROUP, group_raw)
core.insertLocalEntityIntoParentsChildListRaw(group, L.LIST_TYPE_KEYSITE_GROUP, keysite, nil)

core.assessGroupSupplies(group)

-- group.c :: assess_group_supplies: required = bound (100 - level, 0, keysite level)
assert(group_raw.supplies.ammo_supply_level == 80, "ammo " .. group_raw.supplies.ammo_supply_level)
assert(group_raw.supplies.fuel_supply_level == 90, "fuel " .. group_raw.supplies.fuel_supply_level)
assert(#transmitted == 4, "transmissions " .. #transmitted)
assert(transmitted[1].entityIndex == keysite.index and transmitted[1].floatType == core.FloatType.FLOAT_TYPE_AMMO_SUPPLY_LEVEL, "first transmission: the keysite's ammo")

-- update timing: a sleeping group counts down each frame and leaves the update list when both timers expire
local update = core.createLocalEntityRaw(T.ENTITY_TYPE_UPDATE, {})
core.setUpdateEntity(update)
core.setClientServerEntityFloatValue(group, core.FloatType.FLOAT_TYPE_SLEEP, 0.5)
core.setDeltaTime()
core.updateClientServerEntities()
assert(group_raw.sleep == 0.25, "sleep " .. group_raw.sleep)
core.updateClientServerEntities()
assert(group_raw.sleep == 0, "sleep " .. group_raw.sleep)
assert(group.links.update_link.parent == nil, "group should have left the update list")

-- entity lifecycle: a crate created through the real construction path joins its
-- keysite's cargo list and the sector under it, then family destruction undoes it
core.setEntityWorldMapSize(2, 2, 1024)
core.createLocalSectorEntities()
local sent = #transmitted
local crate = core.createClientServerEntity(T.ENTITY_TYPE_CARGO, core.ENTITY_INDEX_DONT_CARE, {
	{ kind = "parent", type = L.LIST_TYPE_CARGO, entity = keysite },
	{ kind = "int_value", type = core.IntType.INT_TYPE_SIDE, value = core.EntitySide.ENTITY_SIDE_BLUE_FORCE },
	{ kind = "vec3d", type = core.Vec3dType.VEC3D_TYPE_POSITION, x = 1500, y = 0, z = 100 },
})
assert(keysite.roots.cargo_root.first_child == crate, "crate should head the keysite's cargo list")
assert(crate.links.sector_link.parent ~= nil, "crate should be in a sector")
assert(transmitted[sent + 1].create == crate.index and transmitted[sent + 1].entityType == T.ENTITY_TYPE_CARGO, "ENTITY_COMMS_CREATE with the created index")
assert(#transmitted[sent + 1].attributes == 3, "three replicated attributes")
core.destroyClientServerEntityFamily(crate)
assert(keysite.roots.cargo_root.first_child == nil, "crate should have left the keysite")
assert(crate.type == T.ENTITY_TYPE_UNKNOWN, "crate entry should be free")
assert(transmitted[sent + 2].destroy == crate.index, "ENTITY_COMMS_DESTROY")

-- keysite cargo: 35 units of ammo materialise three crates in a row beside the
-- keysite (x 0, 3, 6; y 0.5; z 0), newest first in its cargo list; the game
-- status is the zero-initialised UNINITIALISED, which does not hold it back
assert(core.getGameStatus() == core.GameStatusType.GAME_STATUS_UNINITIALISED, "initial game status")
core.setGameStatus(core.GameStatusType.GAME_STATUS_INITIALISED)
sent = #transmitted
core.updateKeysiteCargo(keysite, 35, core.EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO, core.CARGO_AMMO_SIZE)
assert(#transmitted == sent + 3, "three crates created")
local newest = keysite.roots.cargo_root.first_child
assert(newest.data.mob.position.x == 6 and newest.data.mob.position.y == 0.5, "newest crate at x 6, y 0.5")
-- a lower level destroys the oldest crates; nothing is left to create, so the
-- FARP (which consumes ammo) tells its force, whose response (fc_msgs.c) finds
-- no factory, refinery or airbase to supply it: no supply task is asked for
sent = #transmitted
local ok, err = pcall(core.updateKeysiteCargo, keysite, 12, core.EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO, core.CARGO_AMMO_SIZE)
assert(ok, "no supplier: " .. tostring(err))
assert(#transmitted == sent + 2 and transmitted[sent + 1].destroy ~= nil, "two crates destroyed")
assert(keysite.roots.cargo_root.first_child == newest, "the newest crate survives")

-- an airbase supplies itself (it is the closest airbase to its own position):
-- its own crate becomes the cargo, and create_supply_task (Slice 5b) looks for
-- a start keysite: none takes helicopters or transports (landing types 0), so
-- no task is created and nothing is transmitted
keysite.data.sub_type = core.EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_AIRBASE
core.setGameType(core.GameType.GAME_TYPE_CAMPAIGN)
sent = #transmitted
local ok, err = pcall(core.updateKeysiteCargo, keysite, 12, core.EntitySubTypeCargo.ENTITY_SUB_TYPE_CARGO_AMMO, core.CARGO_AMMO_SIZE)
assert(ok, "no start keysite: " .. tostring(err))
assert(#transmitted == sent, "no task transmitted")
assert(#missions == 0, "no mission created")
assert(task_generation[core.EntitySubTypeTask.ENTITY_SUB_TYPE_TASK_SUPPLY + 1].created == 0, "no supply task counted")

print(_VERSION .. ": eech-core.lua smoke test passed")
