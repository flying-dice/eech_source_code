-- TEST-ONLY PUBLIC-HOST OWNERSHIP WITNESS of the physical SAMPLE METADATA at initialisation (EEDCS-51, D3). SOURCE PREPARED: UNBUILT, UNRUN, UNREVIEWED. A script only: no C, no ABI, no provider and no host solver is written here.
--
-- THE QUESTION. The original initialisation read the sample rates from the campaign's own table and opened, read and searched sound.hdr and sound.dat itself. That is now the HOST's: the campaign names its
-- descriptors once, at boot, through the public path (host-owned Lua -> `eech_dc` -> the campaign -> the host's provider callback) as the operation `sound_catalogue`, and takes every rate, original rate and
-- size from the answer (docs/d3-sound-lifetime.md, eech-world/src/env.rs, eech-world/src/sound_data.rs). This witness asks, on the native it is run against, whether the campaign really asks the host and whether the
-- host's ANSWER is what the real sound database shows.
--
-- WHAT IT DOES. It wraps the `host.env_provider` factory BEFORE the delegate boots and counts ONLY the requests whose `op` is `sound_catalogue`; every other request is forwarded UNCHANGED to the original provider.
-- It then runs the HOSTED-INITIALISATION driver `lua/env-sound-init-host.lua` (derived from the original driver, which is NOT edited and refuses a hosted report by design) through `dofile`, so the delegate's argument checks, report
-- validation, raw sidecar and output-failure handling are its own, with no second parser. Its report separates three kinds of fact, and this witness keeps them apart: the HOST'S ANSWER (the `env-sndinit-host-answer` row, a mirror
-- of what the host said), the campaign's OWN file operations (the file-attempt rows, which must be 0 because the host owns the files) and the descriptors' real numbers (`env-sndinit-samples`, the catalogue digest). The delegate's evidence stays on `out` and `<out>.case15.raw` untouched; this witness writes ONE compact separate record of its own on `authority_out`.
--
-- MODES (`mode=`, default healthy):
--   healthy        the request is forwarded to the reference host. REQUIRED: the delegate completed, the host was asked (at least once) and the real report shows what the reference answers for the installation under `root`
--                  `expect=`): `absent` (the installation has no pair: the host's answer says no header, header count 0, nothing matched, every size 0, the catalogue digest the measured original's 6605372942e1ea2c) or `control` (the
--                  measured original's valid-header control: the host's answer says a header of 7 entries, four matches, lightning's first entry 96, thunder matched at 0, digest 8c600fed7309b2fc). In both, the campaign's OWN
--                  header and data open attempts and header-open events are 0 (the measured original's 3 or 4 were the campaign's; those are now the host's operations and are not recreated). Against a native that reads the table
--                  and the files itself the host is asked 0 times: `sound-data-host-capability-missing`.
--   changed        the host answers the lightning sample with rate 44100, original rate 22050 and a matched header entry of length 4321, and the thunder sample with rate and original rate 8000 and a matched entry of
--                  length 0 (the header marked present). REQUIRED: the REAL report shows exactly those numbers (lightning 44100/4321/22050, thunder 8000/0/8000), the host's answer mirrored as a present header, the number of matched
--                  descriptors the answer holds, and a catalogue digest that is neither the healthy digest nor the control's: the numbers are the host's, not the campaign's.
--   withheld       the host raises an error in `sound_catalogue`. REQUIRED: the campaign STOPS at `[sound-data-provider:callback]` (the delegate fails in the engine) and no report row is written.
--   malformed      the host answers a rate of 0. REQUIRED: the campaign stops at `[sound-data-provider:answer]`.
--   short          the host answers one entry fewer than asked. REQUIRED: the campaign stops at `[sound-data-provider:answer]`.
--   incoherent     the host answers a header entry for a descriptor while saying there is no header. REQUIRED: the campaign stops at `[sound-data-provider:answer]`.
--   nothing is applied before the whole answer is checked: a stop is the first and only thing the engine does with a refused answer, so the delegate writes no boot row.
--
-- VERDICTS (errors begin `env-sound-data-host-witness:`): `SETUP FAILURE` -- the delegate failed (healthy, changed) or the stop is not the engine's: NOT a verdict on ownership; `sound-data-host-capability-missing`;
-- `ownership-not-shown` -- the host was asked but the real report does not show the host's answer; `not-stopped` / `wrong-boundary` -- a withheld or malformed answer did not stop the campaign at its marker.
--
-- WHAT IT DOES NOT. It asserts no private type, struct layout or ABI, creates no playback handle, loads no sample data and exercises no renderer, speaker, network or device; it says nothing about the other compiled readers
-- of the sound database or the sample loader (`load_side_dependant_application_sound_samples`), which still reads and may still change the metadata when a sound device exists. It computes no rate or size.
--
-- ARGUMENTS (host.args, as the host executable passes them):
--   eech-world <this script> root=<installation root> out=<delegate JSON Lines path> delegate=<ABSOLUTE path of lua/env-sound-init-host.lua> [mode=healthy|changed|withheld|malformed|short|incoherent] [expect=absent|control]
--                                                                       [scenario=lebanon_retail|georgia_retail] [seconds=<n>] [frame_ms=<n>] [authority_out=<this witness's record path; default <out>.authority.jsonl>]
--   `root`, `out`, `scenario`, `seconds` and `frame_ms` are the delegate's own arguments and are passed through unchanged. One fresh process and root per run.

io.stdout:setvbuf ("no")

local NAME = "env-sound-data-host-witness"
local args = host.args
local function refuse (message) error (NAME .. ": " .. message, 0) end

local delegate_path = args.delegate
if type (delegate_path) ~= "string" or delegate_path == "" then refuse ("delegate=<absolute path of env-sound-init-host.lua> is required") end
local out_path = args.out
if type (out_path) ~= "string" or out_path == "" then refuse ("out=<the delegate's JSON Lines path> is required") end
local mode = args.mode == nil and "healthy" or args.mode
local MODES = {
	healthy = {},
	changed = {},
	withheld = { marker = "[sound-data-provider:callback]" },
	malformed = { marker = "[sound-data-provider:answer]" },
	short = { marker = "[sound-data-provider:answer]" },
	incoherent = { marker = "[sound-data-provider:answer]" },
}
local plan = MODES [mode]
if plan == nil then refuse ("mode must be healthy, changed, withheld, malformed, short or incoherent, got '" .. tostring (mode) .. "'") end
local MARKER = plan.marker
local expect = args.expect == nil and "absent" or args.expect
if expect ~= "absent" and expect ~= "control" then refuse ("expect must be absent or control, got '" .. tostring (expect) .. "'") end
local OP = "sound_catalogue"
local authority_path = args.authority_out
if authority_path == nil then authority_path = out_path .. ".authority.jsonl" end
if type (authority_path) ~= "string" or authority_path == "" or authority_path == out_path then refuse ("authority_out must be a path other than out") end
local raw_path = args.raw_out
if raw_path == nil then raw_path = out_path .. ".case15.raw" end

-- the measured original's values (reference/d3-environment-characterization/sound-init-native-7574 and sound-header-present-native-7574)
local EXPECTED = {
	absent = { present = "0", header_count = "0", matched = "0", lightning_size = "0", thunder_size = "0", nonzero_size = "0", digest = "6605372942e1ea2c" },
	control = { present = "1", header_count = "7", matched = "4", lightning_size = "96", thunder_size = "0", nonzero_size = "3", digest = "8c600fed7309b2fc" },
}

-- ---------------------------------------------------------------------------------------------------------------- the wrapper: forward everything, count only the op, alter only that op's answer
local requests, forwarded, altered = 0, 0, 0
local final_matched, final_header_count = 0, 0
local factory = host.env_provider
if type (factory) ~= "function" then refuse ("this host has no host.env_provider factory") end

local function alter (request, answer)
	if mode == "withheld" then error ("the host withholds its " .. OP .. " answer", 0) end
	if type (answer) ~= "table" or type (answer.entries) ~= "table" then return answer end
	if mode == "malformed" then
		altered = altered + 1
		answer.entries [1].rate = 0
	elseif mode == "short" then
		altered = altered + 1
		answer.entries [#answer.entries] = nil
	elseif mode == "incoherent" then
		altered = altered + 1
		answer.header_present = false
		answer.header_count = 0
		answer.entries [1].matched = true
		answer.entries [1].size = 5
	elseif mode == "changed" then
		for position, entry in ipairs (request.entries) do
			local reply = answer.entries [position]
			if entry.name == "lstrike.pcm" then
				reply.rate, reply.original_rate, reply.matched, reply.size = 44100, 22050, true, 4321
				altered = altered + 1
			elseif entry.name == "thunder.pcm" then
				reply.rate, reply.original_rate, reply.matched, reply.size = 8000, 8000, true, 0
				altered = altered + 1
			end
		end
		answer.header_present = true
		if answer.header_count < 2 then answer.header_count = 2 end
		final_header_count = answer.header_count
		final_matched = 0
		for _, reply in ipairs (answer.entries) do if reply.matched then final_matched = final_matched + 1 end end
	end
	return answer
end

local wrapped = function (...)
	local base = factory (...)
	if type (base) ~= "function" then error (NAME .. ": host.env_provider () did not return a provider function", 0) end
	return function (request, ...)
		if type (request) == "table" and request.op == OP then
			requests = requests + 1
			return alter (request, base (request, ...))
		end
		forwarded = forwarded + 1
		return base (request, ...)
	end
end
local installed, install_error = pcall (function () host.env_provider = wrapped end)
if not installed or host.env_provider ~= wrapped then refuse ("the host table does not accept a wrapped env_provider: " .. tostring (install_error)) end

-- ---------------------------------------------------------------------------------------------------------------- the delegate, exactly as it is
local ran, problem = pcall (dofile, delegate_path)
host.env_provider = factory

local function read_all (path)
	local file = io.open (path, "r")
	if not file then return nil end
	local text = file:read ("*a")
	file:close ()
	return text
end

local function write_record (completed, verdict, extra)
	local file, err = host.output_create_new (authority_path)
	if not file then refuse ("OUTPUT FAILURE: creating " .. authority_path .. " failed: " .. tostring (err) .. " (evidence is created exclusively and never overwritten)") end
	local line = string.format ('{"altered":%d,"completed":%s,"delegate_ran":%s,"expect":"%s","forwarded_other_requests":%d,"mode":"%s","op":"%s","requests":%d,%s"source":"host.env_provider wrapper on the public path","verdict":"%s"}\n',
		altered, completed and "true" or "false", ran and "true" or "false", expect, forwarded, mode, OP, requests, extra or "", verdict)
	local ok, werr = file:write (line)
	if not ok then refuse ("OUTPUT FAILURE: writing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:flush ()
	if not ok then refuse ("OUTPUT FAILURE: flushing " .. authority_path .. " failed: " .. tostring (werr)) end
	ok, werr = file:close ()
	if not ok then refuse ("OUTPUT FAILURE: closing " .. authority_path .. " failed: " .. tostring (werr)) end
end

-- one row of the delegate's raw native report, as `key=value` fields (nil when absent)
local function fields_of (text, prefix)
	for line in string.gmatch (text, "[^\n]+") do
		if string.sub (line, 1, #prefix) == prefix then
			local map = {}
			for key, value in string.gmatch (line, "([%w_]+)=(%S+)") do map [key] = value end
			return map
		end
	end
	return nil
end

local function has (text, fragment) return text ~= nil and string.find (text, fragment, 1, true) ~= nil end

-- ---------------------------------------------------------------------------------------------------------------- withheld, malformed, short, incoherent: the campaign must STOP at the named boundary
if MARKER then
	if ran then
		write_record (true, "not_stopped")
		refuse ("not-stopped: the host answered its '" .. OP .. "' request wrongly (" .. mode .. ", " .. requests .. " request(s)) and the delegate completed: the campaign did not stop at " .. MARKER)
	end
	local delegate_error = tostring (problem)
	if requests == 0 then
		write_record (false, "setup_failure_the_host_was_never_asked")
		refuse ("SETUP FAILURE: the delegate stopped but the host was asked 0 times for '" .. OP .. "': the stop is not the " .. mode .. " answer's: " .. delegate_error)
	end
	if mode ~= "withheld" and altered == 0 then
		write_record (false, "setup_failure_nothing_was_altered")
		refuse ("SETUP FAILURE: the delegate stopped, but the host altered no answer in mode " .. mode .. ": the stop is not the altered answer's: " .. delegate_error)
	end
	if not has (delegate_error, "host sound data") then
		write_record (false, "setup_failure_delegate_error_is_not_the_engine_stop")
		refuse ("SETUP FAILURE: the delegate's error is not its report of the engine's stop at the sample metadata provider: " .. delegate_error)
	end
	if not has (delegate_error, MARKER) then
		write_record (false, "wrong_boundary")
		refuse ("wrong-boundary: the engine stopped at the sample metadata provider, but not at " .. MARKER .. ": " .. delegate_error)
	end
	if has (read_all (out_path), '"event":"boot"') then
		write_record (false, "contradictory_boot_row")
		refuse ("contradictory-result: the delegate wrote its boot row although the campaign is said to have stopped inside boot")
	end
	write_record (false, "stopped_at_boundary", string.format ('"marker":"%s","stopped":true,', MARKER))
	io.stderr:write (string.format ("%s: mode=%s stopped at %s after %d host request(s) for '%s'\n", NAME, mode, MARKER, requests, OP))
	return
end

-- ---------------------------------------------------------------------------------------------------------------- healthy and changed: the delegate must have completed, and the real report must show the host's answer
if not ran then
	write_record (false, "setup_failure_delegate_failed")
	error (tostring (problem), 0)
end
if requests == 0 then
	write_record (true, "sound_data_host_capability_missing")
	refuse ("sound-data-host-capability-missing: the delegate completed and the host was asked 0 times for '" .. OP .. "': the campaign still reads its sample metadata itself")
end
local raw = read_all (raw_path)
local boot = raw and fields_of (raw, "env-sndinit-boot n=0")
local answered = raw and fields_of (raw, "env-sndinit-host-answer n=0")
local core_header = raw and fields_of (raw, "env-sndinit-file kind=header")
local core_data = raw and fields_of (raw, "env-sndinit-file kind=data")
local samples = raw and fields_of (raw, "env-sndinit-samples")
local report = raw and fields_of (raw, "env-sndinit-catalogue n=now when=report")
if not (boot and answered and core_header and core_data and samples and report) then
	write_record (false, "setup_failure_no_report")
	refuse ("SETUP FAILURE: the delegate returned but its native report (" .. raw_path .. ") has no boot, host-answer, file, samples and catalogue rows: there is no ownership verdict")
end
-- THE CAMPAIGN'S OWN FILE OPERATIONS are a separate fact from the host's answer: if the campaign still opened a sound file, or recorded a header open itself, the files are not the host's alone
local core_io = {
	{ "core_header_open_attempts", "0", core_header.attempts }, { "core_data_open_attempts", "0", core_data.attempts },
	{ "core_header_open_events", "0", tostring (tonumber (boot.open_true) + tonumber (boot.open_false)) },
}

local function differences (wanted)
	local wrong = {}
	for _, item in ipairs (wanted) do
		if item [2] ~= item [3] then wrong [#wrong + 1] = item [1] .. " expected " .. tostring (item [2]) .. " observed " .. tostring (item [3]) end
	end
	return wrong
end

local wrong
if mode == "healthy" then
	local want = EXPECTED [expect]
	wrong = differences {
		{ "host_reply_header_present", want.present, answered.present }, { "host_reply_header_count", want.header_count, answered.header_count }, { "host_reply_matched", want.matched, answered.matched },
		{ "lightning_size", want.lightning_size, samples.lightning_size }, { "thunder_size", want.thunder_size, samples.thunder_size }, { "nonzero_size", want.nonzero_size, report.nonzero_size },
		{ "digest_w", want.digest, report.digest_w }, { "lightning_rate", "22050", samples.lightning_rate }, { "thunder_rate", "22050", samples.thunder_rate },
	}
else
	wrong = differences {
		{ "lightning_rate", "44100", samples.lightning_rate }, { "lightning_size", "4321", samples.lightning_size }, { "lightning_orate", "22050", samples.lightning_orate },
		{ "thunder_rate", "8000", samples.thunder_rate }, { "thunder_size", "0", samples.thunder_size }, { "thunder_orate", "8000", samples.thunder_orate },
		{ "host_reply_header_present", "1", answered.present }, { "host_reply_header_count", tostring (final_header_count), answered.header_count }, { "host_reply_matched", tostring (final_matched), answered.matched },
	}
	if report.digest_w == EXPECTED.absent.digest or report.digest_w == EXPECTED.control.digest then
		wrong [#wrong + 1] = "digest_w is still a digest of the unaltered host's answer (" .. tostring (report.digest_w) .. ")"
	end
	if altered ~= 2 then wrong [#wrong + 1] = "the host altered " .. altered .. " answers where it was to alter the lightning and thunder samples" end
end
for _, item in ipairs (core_io) do
	if item [2] ~= item [3] then wrong [#wrong + 1] = item [1] .. " expected " .. item [2] .. " observed " .. tostring (item [3]) .. " (the campaign still performs the file operation the host owns)" end
end
if #wrong > 0 then
	write_record (true, "ownership_not_shown")
	refuse ("ownership-not-shown: the host was asked " .. requests .. " time(s) but the real report does not show its answer: " .. table.concat (wrong, "; "))
end
write_record (true, mode == "healthy" and "host_asked_and_decided" or "host_changed_the_decision", string.format ('"digest_w":"%s",', report.digest_w))
io.stderr:write (string.format ("%s: mode=%s, expect=%s: the host was asked %d time(s) and the real report shows its answer\n", NAME, mode, expect, requests))
