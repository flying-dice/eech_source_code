--
-- Smoke test of the production bundle (build/lua/eech-core.lua) from the point
-- of view of a Lua 5.1 host such as DCS World: plain function calls, Lua table
-- ports, no TypeScript runtime beyond what the bundle carries.
--

assert(_VERSION == "Lua 5.1", "expected Lua 5.1, got " .. tostring(_VERSION))

local core = dofile("build/lua/eech-core.lua")

local transmitted = {}

core.initialiseCampaignCore({
	mobilePhysicalState = {
		getMobilePosition = function(entityIndex) return { x = 0, y = 0, z = 0 } end,
	},
	entityReplication = {
		transmitEntityFloatValue = function(entityIndex, floatType, value)
			transmitted[#transmitted + 1] = { entityIndex = entityIndex, floatType = floatType, value = value }
		end,
	},
})

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
}
local group = core.createLocalEntityRaw(T.ENTITY_TYPE_GROUP, group_raw)
core.insertLocalEntityIntoParentsChildListRaw(group, L.LIST_TYPE_KEYSITE_GROUP, keysite, nil)

core.assessGroupSupplies(group)

-- group.c :: assess_group_supplies: required = bound (100 - level, 0, keysite level)
assert(group_raw.supplies.ammo_supply_level == 80, "ammo " .. group_raw.supplies.ammo_supply_level)
assert(group_raw.supplies.fuel_supply_level == 90, "fuel " .. group_raw.supplies.fuel_supply_level)
assert(#transmitted == 4, "transmissions " .. #transmitted)

-- production policy: reaching an unported message response fails loudly
group_raw.sub_type = core.EntitySubTypeGroup.ENTITY_SUB_TYPE_GROUP_PRIMARY_FRONTLINE
local ok, err = pcall(core.assessGroupSupplies, group)
assert(not ok and tostring(err.message or err):find("response_to_force_low_on_supplies", 1, true), "expected unported message failure")

print(_VERSION .. ": eech-core.lua smoke test passed")
