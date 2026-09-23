--
-- Smoke test of the production bundle (build/lua/eech-core.lua) from the point
-- of view of a Lua 5.1 host such as DCS World: plain function calls, Lua table
-- ports, no TypeScript runtime beyond what the bundle carries.
--

assert(_VERSION == "Lua 5.1", "expected Lua 5.1, got " .. tostring(_VERSION))

local core = dofile("build/lua/eech-core.lua")

local transmitted = {}

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
	},
}, { numberOfEntities = 64 })

local T, L = core.EntityType, core.ListType

local session = core.createLocalEntityRaw(T.ENTITY_TYPE_SESSION, {})
core.setSessionEntityRaw(session)

local force = core.createLocalEntityRaw(T.ENTITY_TYPE_FORCE, { side = core.EntitySide.ENTITY_SIDE_BLUE_FORCE })
core.insertLocalEntityIntoParentsChildListRaw(force, L.LIST_TYPE_FORCE, session, nil)

local keysite = core.createLocalEntityRaw(T.ENTITY_TYPE_KEYSITE, {
	sub_type = core.EntitySubTypeKeysite.ENTITY_SUB_TYPE_KEYSITE_FARP,
	side = core.EntitySide.ENTITY_SIDE_BLUE_FORCE,
	in_use = 1,
	position = { x = 0, y = 0, z = 0 },
	supplies = { ammo_supply_level = 50, fuel_supply_level = 50 },
})
core.insertLocalEntityIntoParentsChildListRaw(keysite, L.LIST_TYPE_KEYSITE_FORCE, force, nil)

local group_raw = {
	sub_type = core.EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_ATTACK_HELICOPTER,
	side = core.EntitySide.ENTITY_SIDE_BLUE_FORCE,
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

-- production policy: reaching an unported message response fails loudly
group_raw.sub_type = core.EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE
local ok, err = pcall(core.assessGroupSupplies, group)
assert(not ok and tostring(err.message or err):find("response_to_force_low_on_supplies", 1, true), "expected unported message failure")

print(_VERSION .. ": eech-core.lua smoke test passed")
