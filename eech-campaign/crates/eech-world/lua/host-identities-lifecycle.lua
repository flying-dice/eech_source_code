-- The module's lifecycle and failure behaviour, observed through the public
-- path (host Lua -> require ("eech_dc")). One case per process: EECH is global
-- state, booted at most once per process, so every case starts from a fresh
-- process (tools/host-identities-lifecycle-windows.ps1 runs and checks them).
--
-- host.args: root (a prepared retail installation), scenario (lebanon_retail or
--            georgia_retail), case (below)
--
-- Each step prints one line to stdout: <step> TAB ok|error TAB <first line of the message>

local args = host.args
local root = assert (args.root, "root=<installation root>")
-- the retail campaigns, as campaign.lua boots them
local scenarios = {
	lebanon_retail = { map = "..\\common\\maps\\map5", campaign = "LEBANON.CHC" },
	georgia_retail = { map = "..\\common\\maps\\map3", campaign = "GEORGIA.CHC" },
}
local scenario = assert (scenarios[args.scenario or "lebanon_retail"], "scenario=lebanon_retail|georgia_retail")

local function config (overrides)
	local c = { install_root = root, map = scenario.map, terrain = host.terrain_provider (root, scenario.map), campaign_directory = "camp01", campaign = scenario.campaign, gunship = "apache", seed = 1, bind_mobile = host.bind_mobile, move_vehicle = host.move_vehicle, observe_loss = host.observe_loss, supply_reached = host.supply_reached, line_of_sight = host.line_of_sight, sense = host.sense, aim = host.aim }
	for k, v in pairs (overrides or {}) do
		if v == false then c[k] = nil else c[k] = v end
	end
	return c
end

local function step (name, f, ...)
	local ok, result = pcall (f, ...)
	local message = ok and "" or (tostring (result):match ("^[^\n]*") or "")
	print (string.format ("%s\t%s\t%s", name, ok and "ok" or "error", message))
	io.stdout:flush ()
	return ok, result
end

local dc = require ("eech_dc")
step ("load", function () assert (type (dc.boot) == "function") end)
step ("prepare", dc.prepare_installation, root)

local function frames (engine, n)
	for _ = 1, n do engine:frame (100) end
end

-- A case that issues its own identities still needs the host to record each
-- vehicle's physical state: the identity the case issues is translated to the one
-- the reference host issued when it bound the same descriptor.
local function own_identities (issue)
	local issued_by_host = {}
	local function bind (descriptor)
		local id = issue (descriptor)
		issued_by_host [id] = host.bind_mobile (descriptor)
		return id
	end
	local function move (request)
		request.host_id = issued_by_host [request.host_id] or request.host_id
		return host.move_vehicle (request)
	end
	return bind, move
end

local cases = {}

-- one boot per process: a second boot is refused and the first engine carries on
function cases.boot_twice ()
	local calls = 0
	local engine, frame_reentry_checked
	local binder, move
	local function issue ()
		calls = calls + 1
		if engine and not frame_reentry_checked then
			frame_reentry_checked = true
			local function rejected (f)
				local ok, message = pcall (f)
				assert (not ok and tostring (message):find ("not valid in the engine's state", 1, true))
			end
			rejected (function () engine:members () end)
			rejected (function () engine:supply_tasks () end)
			rejected (function () engine:roster_stats () end)
			rejected (function () engine:objects () end)
			rejected (function () engine:frame (100) end)
			rejected (function () dc.prepare_installation (root) end)
		end
		return "sequential:" .. calls
	end
	binder, move = own_identities (issue)
	local _
	_, engine = step ("boot", dc.boot, config { bind_mobile = binder, move_vehicle = move })
	step ("frames", frames, engine, 50)
	step ("frame_reentry_guards", function () assert (frame_reentry_checked) end)
	local before_second_calls = calls
	local before_second_members = #engine:members ()
	step ("boot_again", dc.boot, config { bind_mobile = binder, move_vehicle = move })
	step ("roster_not_reset", function ()
		assert (#engine:members () == before_second_members)
		assert (calls == before_second_calls)
	end)
	step ("frames_after", frames, engine, 50)
	step ("roster_consistent", function ()
		assert (#engine:members () == calls)
		assert (calls >= before_second_calls)
	end)
	step ("clock", function () return engine:clock () end)
end

-- the binding checks its arguments before the engine sees them
function cases.bad_arguments ()
	step ("boot_unknown_gunship", dc.boot, config { gunship = "tiger" })
	step ("boot_no_install_root", dc.boot, config { install_root = false })
	step ("boot_no_map", dc.boot, config { map = false })
	step ("boot_no_campaign", dc.boot, config { campaign = false })
	step ("boot_not_a_table", dc.boot, "lebanon")
	local _, engine = step ("boot", dc.boot, config ())
	step ("frame_negative", function () engine:frame (-1) end)
	step ("frame_not_a_number", function () engine:frame ("soon") end)
	step ("frame_zero", function () engine:frame (0) end)
	step ("frames_after", frames, engine, 50)
end

-- inputs the engine itself cannot use
function cases.missing_root ()
	step ("boot_missing_root", dc.boot, config { install_root = root .. "/does-not-exist" })
	step ("boot_after", dc.boot, config ())
end

function cases.missing_map ()
	step ("boot_missing_map", dc.boot, config { map = "..\\common\\maps\\map99" })
	step ("boot_after", dc.boot, config ())
end

function cases.missing_campaign ()
	step ("boot_missing_campaign", dc.boot, config { campaign = "NOSUCH.CHC" })
	step ("boot_after", dc.boot, config ())
end

-- no shutdown or reuse: dropping the engine does not allow another boot, and
-- neither does loading the module again
function cases.no_reuse ()
	local _, engine = step ("boot", dc.boot, config ())
	step ("frames", frames, engine, 50)
	engine = nil
	collectgarbage ()
	collectgarbage ()
	step ("boot_after_collect", dc.boot, config ())
	package.loaded.eech_dc = nil
	local _, reloaded = step ("require_again", require, "eech_dc")
	step ("boot_after_require", reloaded.boot, config ())
end

-- The host capability is required before native engine state is consumed.
function cases.missing_host ()
	step ("boot_without_bind", dc.boot, config { bind_mobile = false })
	local _, engine = step ("boot_after", dc.boot, config ())
	step ("members", function () assert (#engine:members () > 0) end)
end

-- The movement capability is required too, and is checked before engine state is consumed.
function cases.missing_move ()
	step ("boot_without_move", dc.boot, config { move_vehicle = false })
	local _, engine = step ("boot_after", dc.boot, config ())
	step ("members", function () assert (#engine:members () > 0) end)
end

-- A host that issues identities its movement side never recorded: the first step
-- of a vehicle it does not know is a clear failure, not an adopted pose, and the
-- engine is then poisoned like any other fatal error.
function cases.move_unknown_id ()
	local issued = 0
	local function stranger ()
		issued = issued + 1
		return "stranger:" .. issued
	end
	step ("boot", dc.boot, config { bind_mobile = stranger })
	step ("boot_after", dc.boot, config ())
end

-- The callable is invoked and its opaque results become the public durable roster.
-- Re-entry through a different public C entry is refused while boot is active.
function cases.host_identity ()
	local calls, issued = 0, {}
	local binder, move
	local function issue (descriptor)
		calls = calls + 1
		assert (type (descriptor.member_key) == "number" and descriptor.member_key > 0)
		assert (type (descriptor.group_key) == "number" and descriptor.group_key > 0)
		assert (type (descriptor.entity_id) == "number")
		if calls == 1 then
			local ok, message = pcall (dc.prepare_installation, root)
			assert (not ok and tostring (message):find ("not valid in the engine's state", 1, true))
			ok, message = pcall (dc.boot, config { bind_mobile = binder, move_vehicle = move })
			assert (not ok and tostring (message):find ("not valid in the engine's state", 1, true))
		end
		local id = "lifecycle:" .. calls
		issued[id] = true
		return id
	end
	binder, move = own_identities (issue)
	local _, engine = step ("boot", dc.boot, config { bind_mobile = binder, move_vehicle = move })
	step ("reentry_guards", function () assert (calls > 0) end)
	step ("binding_count", function ()
		local members = engine:members ()
		assert (#members == calls and calls > 0)
		for _, member in ipairs (members) do
			assert (issued[member.host_id])
			assert (member.member_key > 0 and member.group_key > 0)
			assert (type (member.initial_alive) == "boolean")
		end
	end)
	step ("objects_compatible", function ()
		local objects = engine:objects ()
		assert (#objects > 0 and objects[1].host_id == nil)
	end)
end

function cases.bind_nil ()
	step ("boot_bind_nil", dc.boot, config { bind_mobile = function () return nil end })
	step ("prepare_after", dc.prepare_installation, root)
	step ("boot_after", dc.boot, config ())
end

function cases.bind_invalid ()
	step ("boot_bind_invalid", dc.boot, config { bind_mobile = function () return string.rep ("x", 129) end })
	step ("prepare_after", dc.prepare_installation, root)
	step ("boot_after", dc.boot, config ())
end

function cases.bind_nul ()
	step ("boot_bind_nul", dc.boot, config { bind_mobile = function () return "bad\0identity" end })
	step ("prepare_after", dc.prepare_installation, root)
	step ("boot_after", dc.boot, config ())
end

function cases.bind_non_utf8 ()
	step ("boot_bind_non_utf8", dc.boot, config { bind_mobile = function () return string.char (0xff, 0xfe) end })
	step ("prepare_after", dc.prepare_installation, root)
	step ("boot_after", dc.boot, config ())
end

function cases.bind_duplicate ()
	step ("boot_bind_duplicate", dc.boot, config { bind_mobile = function () return "duplicate" end })
	step ("prepare_after", dc.prepare_installation, root)
	step ("boot_after", dc.boot, config ())
end

function cases.bind_error ()
	step ("boot_bind_error", dc.boot, config { bind_mobile = function () error ("host refused binding") end })
	step ("prepare_after", dc.prepare_installation, root)
	step ("boot_after", dc.boot, config ())
end

local case = assert (cases[args.case], "case=boot_twice|bad_arguments|missing_root|missing_map|missing_campaign|no_reuse|missing_host|host_identity|bind_nil|bind_invalid|bind_nul|bind_non_utf8|bind_duplicate|bind_error|missing_move|move_unknown_id")
case ()
print ("end")
