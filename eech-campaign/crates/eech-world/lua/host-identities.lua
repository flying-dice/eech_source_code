-- Public-path identity/lifetime audit: host-owned Lua -> eech_dc -> host binder.
-- It adds no campaign observations or decisions and does not alter the existing
-- campaign.lua metrics, diagnostics, structured observations or Tacview stream.
--
-- host.args: root, scenario, hours, frame_ms, sample_every, target_entity,
--            require_reuse=1, require_transfer=1, require_target_released=1

local args = host.args
local root = assert (args.root, "root=<installation root>")
local scenarios = {
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
}
local scenario_name = args.scenario or "lebanon_retail"
local scenario = assert (scenarios[scenario_name], "scenario=georgia_retail|lebanon_retail")
local frame_ms = tonumber (args.frame_ms or "100")
local frames = math.floor (tonumber (args.hours or "3") * 3600000 / frame_ms + 0.5)
local sample_every = tonumber (args.sample_every or "100")
local target_entity = tonumber (args.target_entity or "-1")

local dc = require ("eech_dc")
dc.prepare_installation (root)

local bind_count = 0
local issued_by_id, issued_by_member = {}, {}
local initial_group, groups_seen = {}, {}
local target_member_key
local function binder (descriptor)
	bind_count = bind_count + 1
	local id = host.bind_mobile (descriptor)
	assert (type (id) == "string" and id ~= "")
	assert (not issued_by_id[id], "host reused identity " .. id)
	assert (not issued_by_member[descriptor.member_key], "member was rebound " .. descriptor.member_key)
	issued_by_id[id] = descriptor.member_key
	issued_by_member[descriptor.member_key] = id
	initial_group[descriptor.member_key] = descriptor.group_key
	groups_seen[descriptor.member_key] = { [descriptor.group_key] = true }
	if descriptor.entity_id == target_entity and not target_member_key then target_member_key = descriptor.member_key end
	return id
end

local engine = dc.boot {
	install_root = root,
	map = scenario.map,
	campaign_directory = "camp01",
	campaign = scenario.campaign,
	gunship = "apache",
	seed = 1,
	bind_mobile = binder,
	move_vehicle = host.move_vehicle,
	observe_loss = host.observe_loss,
	supply_reached = host.supply_reached,
	line_of_sight = host.line_of_sight, model = host.model,
	sense = host.sense, aim = host.aim,
}

local latest = {}
local reused_indices = {}
local transfer_members = {}
local function audit ()
	local members = engine:members ()
	assert (#members == bind_count, string.format ("roster %d differs from bind count %d", #members, bind_count))
	local member_keys, host_ids = {}, {}
	for _, member in ipairs (members) do
		assert (issued_by_member[member.member_key] == member.host_id, "member identity changed")
		assert (not member_keys[member.member_key], "duplicate member key")
		assert (not host_ids[member.host_id], "duplicate host identity in roster")
		member_keys[member.member_key], host_ids[member.host_id] = true, true
		latest[member.member_key] = member
		groups_seen[member.member_key][member.group_key] = true
		if member.group_key ~= initial_group[member.member_key] then transfer_members[member.member_key] = true end
		local origins = reused_indices[member.origin_entity_id]
		if not origins then origins = {}; reused_indices[member.origin_entity_id] = origins end
		origins[member.host_id] = true
	end
end

audit ()
for frame = 1, frames do
	engine:frame (frame_ms)
	if frame % sample_every == 0 then audit () end
end
audit ()

local transfers, reused, durable, target
transfers, reused, durable = 0, 0, 0
for _ in pairs (transfer_members) do transfers = transfers + 1 end
for _, identities in pairs (reused_indices) do
	local count = 0
	for _ in pairs (identities) do count = count + 1 end
	if count > 1 then reused = reused + 1 end
end
for _, member in pairs (latest) do
	if not member.physical_present then durable = durable + 1 end
end
if target_member_key then target = latest[target_member_key] end

print (string.format ("binds\t%d", bind_count))
print (string.format ("members\t%d", #engine:members ()))
print (string.format ("transfers\t%d", transfers))
print (string.format ("reused_indices\t%d", reused))
print (string.format ("durable_removed\t%d", durable))
local stats = engine:roster_stats ()
print (string.format ("duplicate_comparisons\t%d", stats.duplicate_comparisons))
print (string.format ("duplicate_scan_seconds\t%.6f", stats.duplicate_scan_seconds))
if target then
	print (string.format ("target\t%s\tphysical_present=%s\tin_group=%s", target.host_id, tostring (target.physical_present), tostring (target.in_group)))
end
io.stdout:flush ()

if args.require_reuse == "1" then assert (reused > 0, "no physical entity-index reuse observed") end
if args.require_transfer == "1" then assert (transfers > 0, "no live group transfer observed") end
if target_entity >= 0 then
	assert (target, "target entity has no durable roster record")
	assert (not target.in_group, "target entity remains a campaign group member")
	if args.require_target_released == "1" then assert (not target.physical_present, "target wreck has not been removed") end
end

print ("result\tok")
