//! Boundary checks of `lua/sector-height-witness.lua` (EEDCS-49, D3, Task49): the host-owned driver of the two REQUIRED witnesses ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 (M5) and
//! ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 (M6). They run the REAL driver, unmodified, under a real Lua 5.1 state, the REAL exclusive-output host function and the REAL reference arithmetic profile (SSE
//! round-toward-zero, selected and put back by the harness), with ONE scripted collaborator standing in for the campaign DLL (`require ("eech_dc")`: a boot, a frame loop and the session method
//! `sector_height_witness_save`) and a scripted host provider. The collaborator makes the calls the real campaign makes, in the order its source makes them (a task leader's step reads the sector word twice with
//! nothing between, then asks the look-ahead and hands the minimum height to the host's move; the session save packs every sector, and RETRIES the whole pack when its buffer grows; the load restores every saved word
//! before the first insertion) and checks every reply the way the C bridge does (mb_hostentity.h check_sector_height_reply), so an altered reply stops it with the marker the real engine stops with.
//!
//! WHAT THIS IS NOT: campaign authority. Nothing here runs the guarded `fw_move.c` branch, `store_session`, `pack_session` or `unpack_session`; the first real observation of each is the native run (docs/
//! d3-sector-height-production-contract.md section 12). These checks show that the DRIVER credits only the eligible intervention it made, never a fatal before it, never an unreached case, consumes only whole successful
//! reference reports and the exact reply grammar, and that its controls and every single-fault variant of them end as stated. The scripted collaborator's hand-off arithmetic is computed in Rust single precision
//! UNDER the actually selected reference profile (`HANDOFF`) and is a stand-in, not a copy of the campaign's code.
//!
//! The helper block of single-precision words in the driver is the SAME text as entity-authority.lua's (the driver runs under a hardware divide-by-zero trap natively): compared below.

use eech_world::reference_arithmetic;
use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};

const DRIVER: &str = include_str!("../lua/sector-height-witness.lua");
const ENTITY_AUTHORITY: &str = include_str!("../lua/entity-authority.lua");

/// the scripted collaborator: the campaign, its host provider and the reference host's movement, parameterised by the case table `P`
const COLLABORATOR: &str = r#"
LOG = {}
HELD = {}
local PHASE = { first = "0x00000000", entry = "0x3f800000", ["end"] = "0x40000000", resolved = "0x40400000", insert = "0x40800000", reset = "0x40c00000", read = "0x41880000", restore = "0x41900000" }
local PHASE_NAME = {}
for name, pattern in pairs (PHASE) do PHASE_NAME [pattern] = name end
local ZERO = "0x00000000"
local function note (what) LOG [#LOG + 1] = what end

local function split (text, separator)
	local t = {}
	for token in string.gmatch (text, "[^" .. separator .. "]+") do t [#t + 1] = token end
	return t
end
-- the facts of a question; a read of the leader's sector can be damaged on its Nth ask (P.bad_request) to test the exact request grammar
local LEADER_ASKS = 0
local function facts (phase, x, z, stated)
	local w = {}
	for i = 1, 16 do w [i] = ZERO end
	w [1] = stated or ZERO
	w [12] = PHASE [phase]
	w [13], w [14] = x, z
	local text = table.concat (w, ",")
	if phase == "read" and P.leader_sector and x == P.leader_sector [1] and z == P.leader_sector [2] then
		LEADER_ASKS = LEADER_ASKS + 1
		if P.bad_request and LEADER_ASKS == (P.bad_request_nth or 1) then
			if P.bad_request == "identity" then return text .. " extra" end
			if P.bad_request == "comma" then return (string.gsub (text, ",", ",,", 1)) end
			if P.bad_request == "nonzero" then return (string.gsub (text, ZERO, "0x3f800000", 1)) end
		end
	end
	return text
end

local LEADER_REPLIES = 0
-- the host's provider (scripted): reads and restores of sector words, scans, inserts with the strict-greater raise
function PROVIDER (op, scene, depth, kind, definition, key)
	note ("model " .. tostring (op))
	if op ~= "entity_scan" then return { result = "ok" } end
	local w = split (string.match (key, "^(%S+)"), ",")
	local phase = PHASE_NAME [w [12]]
	local k = w [13] .. "," .. w [14]
	if phase == "read" then
		local word = HELD [k] or (P.held and P.held [k]) or ZERO
		local reply = { result = "height", p = w [13] .. "," .. w [14] .. "," .. word }
		if P.leader_sector and k == P.leader_sector [1] .. "," .. P.leader_sector [2] then
			LEADER_REPLIES = LEADER_REPLIES + 1
			if P.reply and LEADER_REPLIES == (P.reply_nth or 1) then
				if P.reply == "trailing" then reply.p = reply.p .. "," end
				if P.reply == "empty" then reply.p = w [13] .. ",," .. word end
				if P.reply == "name" then reply.name = "1" end
				if P.reply == "extra" then reply.p = reply.p .. ",0x00000000" end
				if P.reply == "result" then reply.result = "nope" end
			end
		end
		return reply
	elseif phase == "restore" then
		HELD [k] = w [1]
		local stored = w [1]
		if P.stored_wrong == k then stored = string.format ("0x%08x", tonumber (string.sub (w [1], 3), 16) + 1) end
		local reply = { result = "restored", p = w [13] .. "," .. w [14] .. "," .. stored }
		if P.seed_reply == "trailing" then reply.p = reply.p .. "," end
		return reply
	elseif phase == "insert" then
		-- scripted insert: the sector named in facts 12 and 13 is raised to the sum in fact 0 when it is strictly greater (a NaN compares false)
		local held = HELD [k] or ZERO
		if P.lowering == k then HELD [k] = "0xc1200000"
		elseif P.nan_change == k then HELD [k] = "0x3f800000"
		elseif F32_GT (w [1], held) then HELD [k] = w [1] end
		return { result = "placed" }
	elseif phase == "first" then
		return { result = "first", name = "1", p = w [13] .. "," .. w [14] }
	elseif phase == "end" then
		return { result = "done_none" }
	end
	return { result = "ok" }
end

-- the campaign's read of a sector word, checked like mb_hostentity.h (the bridge's list check first, then the state, the sector echoed, and the stored word of a restore)
local function ask (config, phase, x, z, stated)
	local ok, answer = pcall (config.model, "entity_scan", 0, nil, nil, nil, facts (phase, x, z, stated))
	if not ok then error ("host model: [model-provider:" .. (P.callback_marker or "entity_scan.callback") .. "] " .. tostring (answer), 0) end
	if P.lenient then
		local p = split (answer.p, ",")
		return p [3] or ZERO, answer
	end
	if type (answer) ~= "table" then error ("host model: [model-provider:entity_scan.answer] the answer is not a table", 0) end
	local p = split (type (answer.p) == "string" and answer.p or "", ",")
	if #p ~= 3 then error ("host model: [model-provider:entity_scan.answer] p must be three patterns", 0) end
	local wanted = phase == "restore" and "restored" or "height"
	if answer.result ~= wanted then error ("host model: [model-provider:entity_scan.state] the host decided " .. tostring (answer.result) .. " where " .. wanted .. " is allowed", 0) end
	if p [1] ~= x or p [2] ~= z then error ("host model: [model-provider:entity_scan.sector_height] another sector than the one asked about", 0) end
	if stated and p [3] ~= stated then error ("host model: [model-provider:entity_scan.sector_height] the host stored a word that is not the word stated", 0) end
	return p [3], answer
end

-- the reference host's movement (scripted): the physical answer depends on the minimum height handed over (half of it, exact) unless the case fixes it
function REFERENCE_MOVE (step)
	note ("move " .. tostring (step.mode))
	if step.mode == "fixed_wing_lookahead" then
		if P.lookahead_nil then return nil end
		return { position = { 10, 3000, 20 } }
	end
	local y = 0
	if step.fixed_wing_terrain then y = step.fixed_wing_terrain.min_height * 0.5 end
	if P.host_y_const then y = P.host_y_const end
	if P.answer_missing then return { motion_vector = { 0, 0, 0 } } end
	return { position = { 10, y, 20 } }
end

-- one task leader's step: the wrapper read, the navigation read, then (with a hand-off) the look-ahead and the move carrying fixed_wing_terrain, all of ONE caller
local function leader_step (config, window, with_handoff)
	if P.break_at_window ~= nil and P.break_at_window == window then error ("host model: [model-provider:" .. (P.break_marker or "entity_scan.callback") .. "] the scripted campaign stops here", 0) end
	local x, z = P.leader_sector [1], P.leader_sector [2]
	-- the original-solver profile consumes its own legacy word: the host is never asked
	local consumed = ZERO
	if not P.original then
		ask (config, "read", x, z)
		consumed = ask (config, "read", x, z)
	end
	if with_handoff then
		local id = P.caller or "ac1"
		config.move_vehicle { mode = "fixed_wing_lookahead", host_id = P.lookahead_id or id }
		if P.between == "bind" then config.bind_mobile {}
		elseif P.between == "model" then config.model ("scene", 1)
		elseif P.between == "move" then config.move_vehicle { mode = "move", host_id = "gv1" } end
		local min_height = HANDOFF (consumed, P.elevation, P.wp_alt)
		if P.ignores_word then min_height = HANDOFF (ZERO, P.elevation, P.wp_alt) end
		if P.bump then min_height = min_height + P.bump end
		local step = {
			mode = "fixed_wing_move", host_id = P.move_id or id, phase = 1, elevation = P.elevation, pose = { position = { 10, 3000, 20 } },
			fixed_wing_terrain = { min_height = min_height, guide_position = { 0, 0, 0 }, sample_point = { 0, 0 }, sample_elevation = 0, site_y = 0, cg = 0 },
		}
		if P.missing == "phase" then step.phase = nil elseif P.missing == "elevation" then step.elevation = nil elseif P.missing == "pose" then step.pose = nil elseif P.missing == "id" then step.host_id = nil end
		config.move_vehicle (step)
	else
		-- a wrapper-only call: nothing hands a height over; the next host callback is another vehicle's
		config.move_vehicle { mode = "move", host_id = "gv1" }
	end
end

local frame_number = 0
local function scan_frame (config)
	-- a scan: BEGIN, a FIRST read of a sector inside it, END
	config.model ("entity_scan", 0, nil, nil, nil, facts ("first", ZERO, ZERO))
	ask (config, "read", "0x40400000", "0x40400000")
	config.model ("entity_scan", 0, nil, nil, nil, facts ("end", ZERO, ZERO))
end

-- the real writer's order: pack_session reads EVERY sector, and store_session RETRIES the whole pack with a larger buffer while pack_session reports an overflow
local function pack (config)
	local attempts = P.pack_attempts or 1
	for attempt = 1, attempts do
		if P.pack_damage and P.pack_damage.attempt == attempt then HELD [P.pack_damage.key] = P.pack_damage.word end
		for index, sector in ipairs (P.map) do
			local skipped = (P.pack_omit == attempt and index == #P.map) or (P.pack_short_first and attempt == 1 and index > 6)
			if not skipped then ask (config, "read", sector [1], sector [2]) end
			if P.pack_twice and attempt == 1 and index == 1 then ask (config, "read", sector [1], sector [2]) end
		end
	end
end

local function load_boot (config)
	local function restore (sector)
		local word = P.saved_words [sector [1] .. "," .. sector [2]]
		if P.restore_override and P.restore_override [1] == sector [1] .. "," .. sector [2] then word = P.restore_override [2] end
		ask (config, "restore", sector [1], sector [2], word)
	end
	local function insert (item) config.model ("entity_scan", 0, nil, nil, nil, facts ("insert", item [1], item [2], item [3])) end
	config.model ("entity_scan", 0, nil, nil, nil, facts ("reset", ZERO, ZERO))
	if P.insert_first then insert (P.inserts [1]) end
	-- the BASELINE population of the supported boot, built before the saved state is unpacked (the actual native load had 282933 of them)
	for _, item in ipairs (P.pre_inserts or {}) do insert (item) end
	if not P.no_restore then
		for index, sector in ipairs (P.map) do
			if P.skip_restore ~= index then restore (sector) end
			if P.dup_restore == index then restore (sector) end
			if P.insert_between == index then insert (P.inserts [1]) end
		end
	end
	if not P.no_inserts then
		for _, item in ipairs (P.inserts) do insert (item) end
	end
end

package.preload ["eech_dc"] = function ()
	return {
		prepare_installation = function () note ("prepare") end,
		boot = function (config)
			note ("boot " .. tostring (config.campaign))
			CONFIG = config
			if P.family == "load" then load_boot (config) end
			local engine = {
				frame = function ()
					frame_number = frame_number + 1
					if P.family ~= "clearance" then return end
					local action = P.frames [frame_number]
					if action == "scan" then scan_frame (CONFIG)
					elseif action == "step" then leader_step (CONFIG, P.window_of [frame_number], false)
					elseif action == "handoff" then leader_step (CONFIG, P.window_of [frame_number], true)
					elseif action == "split" then ask (CONFIG, "read", P.leader_sector [1], P.leader_sector [2]); CONFIG.bind_mobile {}; ask (CONFIG, "read", P.leader_sector [1], P.leader_sector [2]); CONFIG.move_vehicle { mode = "move", host_id = "gv1" }
					end
				end,
			}
			if not P.no_save_method then
				engine.sector_height_witness_save = function (_, name)
					note ("save " .. name)
					if P.save_refused then error ("a saved file of that name exists", 0) end
					pack (CONFIG)
					return P.save_line and string.format (P.save_line, name) or string.format ("sh-witness-save name=%s bytes=4096 version=7\n", name)
				end
			end
			return engine
		end,
	}
end
"#;

struct Outcome {
    result: Result<(), String>,
    report: Option<String>,
    log: Vec<String>,
}

static SERIAL: AtomicUsize = AtomicUsize::new(0);

fn temp_path(stem: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "sector-height-witness-{stem}-{}-{}.txt",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ))
}

/// the thread's own SSE word, put back when the run ends so a failure cannot leak the reference profile into another test
struct RestoreMxcsr(u32);
impl Drop for RestoreMxcsr {
    fn drop(&mut self) {
        reference_arithmetic::write_mxcsr(self.0);
    }
}

fn single_from_bits(text: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(text.trim_start_matches("0x"), 16).unwrap())
}

/// run the real driver with the host args `args` (a Lua table body; `@OUT@` is the report path) and the case table `p` (a Lua table body)
fn run(args: &str, p: &str) -> Outcome {
    let out = temp_path("report");
    let _ = std::fs::remove_file(&out);
    run_to(args, p, &out)
}

fn run_to(args: &str, p: &str, out: &std::path::Path) -> Outcome {
    let _restore = RestoreMxcsr(reference_arithmetic::current_mxcsr());
    let out_path = out.to_string_lossy().replace('\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .set(
            "F32_GT",
            lua.create_function(|_, (a, b): (String, String)| Ok(single_from_bits(&a) > single_from_bits(&b)))
                .unwrap(),
        )
        .unwrap();
    lua.globals()
        .set(
            "HANDOFF",
            lua.create_function(|_, (word, elevation, wp_alt): (String, f64, f64)| {
                // the original expressions in single precision, executed under the profile the driver actually selected: max (altitude, max (terrain, word - terrain)); a scripted stand-in
                assert_eq!(
                    reference_arithmetic::current_mxcsr() & reference_arithmetic::MXCSR_ROUNDING_CONTROL,
                    reference_arithmetic::MXCSR_TOWARD_ZERO,
                    "the scripted campaign ran outside the selected reference profile"
                );
                let word = single_from_bits(&word);
                let (elevation, wp_alt) = (elevation as f32, wp_alt as f32);
                let after_word = elevation.max(word - elevation);
                Ok(f64::from(wp_alt.max(after_word)))
            })
            .unwrap(),
        )
        .unwrap();
    lua.globals()
        .set(
            "BITS",
            lua.create_function(|_, value: f64| Ok(format!("0x{:08x}", (value as f32).to_bits()))).unwrap(),
        )
        .unwrap();
    lua.load(format!("P = {{ {p} }}")).exec().expect("the case table is valid Lua");
    lua.load(COLLABORATOR).set_name("collaborator").exec().expect("the collaborator is valid Lua");
    let args = args.replace("@OUT@", &out_path);
    lua.load(format!(
        r#"
        host = {{
            args = {{ {args} }},
            log = function () end,
            terrain_provider = function () return function () end end,
            env_provider = function () return function () end end,
            model = function (...) return PROVIDER (...) end,
            bind_mobile = function () end, move_vehicle = function (step) return REFERENCE_MOVE (step) end, observe_loss = function () end,
            supply_reached = function () end, line_of_sight = function () end, sense = function () end, aim = function () end,
        }}
        "#
    ))
    .set_name("host")
    .exec()
    .expect("the host table is valid Lua");
    let host: Table = lua.globals().get("host").unwrap();
    // the REAL profile selection of the reference host (src/main.rs host.use_reference_arithmetic): SSE round-toward-zero, read back
    host.set(
        "use_reference_arithmetic",
        lua.create_function(|_, ()| {
            reference_arithmetic::select_sse_toward_zero().map_err(mlua::Error::runtime)?;
            Ok(())
        })
        .unwrap(),
    )
    .unwrap();
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(DRIVER).set_name("sector-height-witness.lua").exec().map_err(|e| e.to_string());
    let log: Vec<String> = lua.globals().get("LOG").unwrap();
    let report = std::fs::read_to_string(out).ok();
    Outcome { result, report, log }
}

// ---------------------------------------------------------------------------------------------------------------- shared fixtures

fn bits(value: f32) -> String {
    format!("0x{:08x}", value.to_bits())
}

const CLEARANCE_P: &str = "family = 'clearance', leader_sector = { BITS (7), BITS (9) }, elevation = 1500.5, wp_alt = 1000.0, \
     frames = { 'scan', 'step', 'handoff', 'handoff' }, window_of = { [2] = 1, [3] = 2, [4] = 3 }";
const ORIGINAL_FRAMES: &str = "original = true, frames = { 'step', 'handoff' }";
const CLEARANCE_ARGS: &str = "root = 'x', out = '@OUT@', hours = '0.0002'";

fn clearance(mode_and_extra_args: &str, extra_p: &str) -> Outcome {
    run(&format!("{CLEARANCE_ARGS}, {mode_and_extra_args}"), &format!("{CLEARANCE_P}, {extra_p}"))
}

fn report(outcome: &Outcome) -> String {
    outcome.report.clone().expect("the report exists")
}

fn line_of<'a>(report: &'a str, prefix: &str) -> &'a str {
    report
        .lines()
        .find(|l| l.starts_with(prefix))
        .unwrap_or_else(|| panic!("no {prefix} line in:\n{report}"))
}

fn assert_failed(outcome: &Outcome, status: &str, needle: &str) {
    let error = outcome.result.as_ref().expect_err(&format!("a {status} run must end non-zero"));
    assert!(error.contains(status) && error.contains(needle), "wanted {status} / {needle}, got: {error}");
    let report = report(outcome);
    assert!(line_of(&report, "sh-witness-verdict").contains(&format!("status={status}")), "{report}");
}

fn assert_refused_before_any_effect(outcome: &Outcome, needle: &str) {
    let error = outcome.result.as_ref().expect_err("the arguments must be refused");
    assert!(error.contains(needle), "wanted {needle}, got: {error}");
    assert!(
        outcome.report.is_none() && outcome.log.is_empty(),
        "no side effect before the arguments are accepted: {:?}",
        outcome.log
    );
}

fn path_text(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn remove(paths: &[&std::path::Path]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
    }
}

/// the original-solver characterisation and the hosted control paired with it, both whole successful reports kept for the runs that consume them
struct Paired {
    original: std::path::PathBuf,
    control: std::path::PathBuf,
}

fn paired(extra_p: &str) -> Paired {
    let original = temp_path("original");
    let control = temp_path("control");
    let _ = std::fs::remove_file(&original);
    let _ = std::fs::remove_file(&control);
    let o = run_to(
        &format!("{CLEARANCE_ARGS}, mode = 'clearance_original'"),
        &format!("{CLEARANCE_P}, {ORIGINAL_FRAMES}, {extra_p}"),
        &original,
    );
    o.result.as_ref().expect("the original profile run passes");
    let c = run_to(
        &format!("{CLEARANCE_ARGS}, mode = 'clearance_control', original = '{}'", path_text(&original)),
        &format!("{CLEARANCE_P}, {extra_p}"),
        &control,
    );
    c.result.as_ref().expect("the hosted control paired with it passes");
    Paired { original, control }
}

impl Paired {
    fn intervene(&self, mode: &str, extra_p: &str) -> Outcome {
        clearance(&format!("mode = '{mode}', window = 2, control = '{}'", path_text(&self.control)), extra_p)
    }
    fn clean(&self) {
        remove(&[&self.original, &self.control]);
    }
}

fn damaged(report: &str, how: &str) -> String {
    let mut lines: Vec<String> = report.lines().map(str::to_string).collect();
    let position = |lines: &[String], prefix: &str| lines.iter().position(|l| l.starts_with(prefix)).unwrap_or_else(|| panic!("no {prefix} line"));
    match how {
        "no_verdict" => {
            lines.pop();
        }
        "failed" => {
            let at = lines.len() - 1;
            lines[at] = lines[at].replace("status=PASS", "status=FAILURE");
        }
        "duplicate_record" => {
            let at = lines
                .iter()
                .position(|l| l.starts_with("sh-witness-handoff ") || l.starts_with("sh-witness-save "))
                .expect("a single record");
            let copy = lines[at].clone();
            lines.insert(at + 1, copy);
        }
        "duplicate_row" => {
            let at = position(&lines, "sh-save-word ");
            let copy = lines[at].clone();
            lines.insert(at + 1, copy);
        }
        "other_setup" => {
            lines[0] = lines[0].replace(" seed=1 ", " seed=2 ");
        }
        "unknown" => {
            let at = lines.len() - 1;
            lines.insert(at, "sh-witness-bogus x=1".to_string());
        }
        "fatal" => {
            let at = lines.len() - 1;
            lines.insert(at, "sh-witness-fatal where=frame marker=none line=stopped".to_string());
        }
        "empty_line" => lines.insert(1, String::new()),
        "wrong_mode" => {
            lines[0] = lines[0]
                .replace("mode=clearance_original", "mode=clearance_control")
                .replace("mode=save", "mode=load");
        }
        "no_counts" => {
            let at = position(&lines, "sh-witness-counts");
            lines.remove(at);
        }
        "schema1" => lines[0] = lines[0].replace("schema=3", "schema=1"),
        "schema2" => lines[0] = lines[0].replace("schema=3", "schema=2"),
        "no_pack" => {
            let at = position(&lines, "sh-witness-pack ");
            lines.remove(at);
        }
        "gap" => {
            let at = position(&lines, "sh-save-word i=3 ");
            lines.remove(at);
        }
        "bad_word" => {
            let at = position(&lines, "sh-save-word i=3 ");
            lines[at] = lines[at].replace("word=0x", "word=0x,");
        }
        "wrong_profile" => {
            lines[0] = lines[0]
                .replace("profile=original-solver", "profile=hosted")
                .replace("profile=witness-feature", "profile=hosted")
        }
        "profile_original" => lines[0] = lines[0].replace("profile=hosted", "profile=original-solver"),
        "no_trajectory" => {
            let at = position(&lines, "sh-witness-trajectory");
            lines.remove(at);
        }
        "unpaired" => {
            let at = position(&lines, "sh-witness-original");
            lines.remove(at);
        }
        other => {
            // `<verb>_<record>_<key>[=<value>]`: drop a key, set a key, or add a second copy of a key, in one record (handoff, identity, counts, trajectory)
            let (verb, rest) = other.split_once('_').unwrap_or_else(|| panic!("unknown damage {other}"));
            let (record, key) = rest.split_once('_').unwrap_or_else(|| panic!("unknown damage {other}"));
            let prefix = match record {
                "handoff" => "sh-witness-handoff ",
                "identity" => "sh-witness-identity ",
                "counts" => "sh-witness-counts ",
                "trajectory" => "sh-witness-trajectory ",
                _ => panic!("unknown damage {other}"),
            };
            match verb {
                "drop" => edit_token(&mut lines, prefix, key, None),
                "set" => {
                    let (key, value) = key.split_once('=').unwrap_or_else(|| panic!("unknown damage {other}"));
                    edit_token(&mut lines, prefix, key, Some(value))
                }
                "dup" => {
                    let at = position(&lines, prefix);
                    lines[at].push_str(&format!(" {key}"));
                }
                _ => panic!("unknown damage {other}"),
            }
        }
    }
    lines.join("\n") + "\n"
}

/// drop (None) or replace (Some) the one `key=value` token of the first line starting with `prefix`; the key must exist, so a damage never silently does nothing
fn edit_token(lines: &mut [String], prefix: &str, key: &str, value: Option<&str>) {
    let at = lines.iter().position(|l| l.starts_with(prefix)).unwrap_or_else(|| panic!("no {prefix} line"));
    let mut seen = false;
    let tokens: Vec<String> = lines[at]
        .split(' ')
        .filter_map(|token| match token.split_once('=') {
            Some((k, _)) if k == key => {
                seen = true;
                value.map(|v| format!("{key}={v}"))
            }
            _ => Some(token.to_string()),
        })
        .collect();
    assert!(seen, "no {key} token in the {prefix} record: {}", lines[at]);
    lines[at] = tokens.join(" ");
}

fn write_temp(text: &str) -> std::path::PathBuf {
    let path = temp_path("damaged");
    std::fs::write(&path, text).unwrap();
    path
}

/// the largest single not above a positive double (the truncation toward zero of the exact value)
fn truncate_to_single(value: f64) -> f32 {
    let nearest = value as f32;
    if f64::from(nearest) > value {
        f32::from_bits(nearest.to_bits() - 1)
    } else {
        nearest
    }
}

/// an elevation, found by stepping single precision values from `start`, for which `word - elevation` is exact in double and NOT exact in single, and for which round-to-nearest and round-toward-zero differ
fn elevation_where_nearest_and_toward_zero_differ(word: f32, start: f32) -> f32 {
    let mut elevation = start;
    for _ in 0..4096 {
        let exact = f64::from(word) - f64::from(elevation);
        let down = truncate_to_single(exact);
        let up = f32::from_bits(down.to_bits() + 1);
        if f64::from(down) != exact && f64::from(up) - exact < exact - f64::from(down) {
            return elevation;
        }
        elevation = f32::from_bits(elevation.to_bits() + 1);
    }
    panic!("no elevation near {start} has a non-exact difference whose nearest and toward-zero values differ");
}

// ---------------------------------------------------------------------------------------------------------------- the float helper block is the one the native host already proved

#[test]
fn the_single_precision_helper_block_is_the_same_text_as_the_one_the_entity_authority_driver_runs() {
    let block = |text: &str| -> Vec<String> {
        let begin = text.find("-- BEGIN words").expect("the words marker");
        let end = text.find("-- END words").expect("the words end marker");
        text[begin..end].lines().map(|l| l.trim_end().to_string()).collect()
    };
    assert_eq!(
        block(DRIVER),
        block(ENTITY_AUTHORITY),
        "the helper block of the witness driver must be the entity-authority one, byte for byte"
    );
    // nothing divides by a value that can be zero: the only `/` of the code is the frame count over the positive frame_ms
    let divisions: Vec<&str> = DRIVER
        .lines()
        .filter(|l| !l.trim_start().starts_with("--"))
        .filter(|l| {
            let code = l.split("--").next().unwrap_or("");
            let mut in_string = false;
            for c in code.chars() {
                if c == '"' {
                    in_string = !in_string;
                } else if c == '/' && !in_string {
                    return true;
                }
            }
            false
        })
        .collect();
    assert_eq!(divisions.len(), 1, "{divisions:?}");
    assert!(divisions[0].contains("frame_ms"), "{divisions:?}");
}

#[test]
fn the_driver_states_a_restore_only_to_its_own_provider_for_seeding_and_never_packs_unpacks_or_restores_for_the_campaign() {
    let code: Vec<&str> = DRIVER.lines().filter(|l| !l.trim_start().starts_with("--")).collect();
    let code_text = code.join("\n");
    for forbidden in ["pack_session", "unpack_session", "store_session", "campaign_model", ".model_restore"] {
        assert!(!code_text.contains(forbidden), "the driver must not call or name {forbidden} in code");
    }
    // the one place a restore question (phase 18) is BUILT is the seeding of the host's own provider; the restores the campaign makes are only observed
    assert_eq!(
        code_text.matches("\"0x41900000\"").count(),
        2,
        "the PHASE_OF entry and the seeding restore only"
    );
    let seeding = code
        .iter()
        .position(|l| l.contains("local function seed_provider"))
        .expect("the seeding function");
    let restore_build = code.iter().position(|l| l.contains("facts_text (\"0x41900000\"")).expect("the seeding restore");
    assert!(
        restore_build > seeding && restore_build < seeding + 6,
        "the restore question is built only inside seed_provider"
    );
    assert_eq!(code_text.matches("engine:sector_height_witness_save (").count(), 1);
    // the host's own provider is delegated to, never replaced: the driver assigns nothing to host.model
    assert!(!code_text.contains("host.model = ") && !code_text.contains("host.model="));
    // the one rounding of the clearance expression is the accepted toward-zero one: no round-to-nearest helper remains
    assert!(
        !code_text.contains("rem == 0.5") && !code_text.contains("ties"),
        "no nearest-even rounding may remain in code"
    );
}

// ---------------------------------------------------------------------------------------------------------------- M5: the guarded clearance

#[test]
fn the_original_profile_characterisation_comes_first_and_the_paired_control_must_equal_it_field_for_field() {
    let original_out = temp_path("original");
    let _ = std::fs::remove_file(&original_out);
    let o = run_to(
        &format!("{CLEARANCE_ARGS}, mode = 'clearance_original'"),
        &format!("{CLEARANCE_P}, {ORIGINAL_FRAMES}"),
        &original_out,
    );
    o.result.as_ref().expect("the original profile run passes");
    let r = report(&o);
    assert!(
        line_of(&r, "sh-witness-identity").starts_with(
            "sh-witness-identity schema=3 mode=clearance_original profile=original-solver scenario=lebanon_retail seed=1 frame_ms=100 hours=0.0002 window=none"
        ),
        "{r}"
    );
    // the EFFECTIVE setup, never a root path: the scenario's own campaign script and map
    assert!(
        line_of(&r, "sh-witness-identity").contains(r" campaign=LEBANON.CHC map=..\common\maps\map5 name=none"),
        "{r}"
    );
    assert!(
        line_of(&r, "sh-witness-counts")
            .contains("windows=0 windows_without_handoff=0 scan_reads=0 height_reads=0 altered_windows=0 altered_reads=0 handoffs=1"),
        "{r}"
    );
    let handoff = line_of(&r, "sh-witness-handoff");
    assert!(handoff.starts_with("sh-witness-handoff ordinal=1 window=0 reads=0 sector=none"), "{handoff}");
    assert!(
        handoff.contains(&format!("host_id=ac1 elevation={} min_height={} ", bits(1500.5), bits(1500.5))),
        "{handoff}"
    );
    assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{r}");
    // the same setup hosted, PAIRED: the first hand-off equals the original profile's, field for field
    let control_out = temp_path("control");
    let _ = std::fs::remove_file(&control_out);
    let o = run_to(
        &format!("{CLEARANCE_ARGS}, mode = 'clearance_control', original = '{}'", path_text(&original_out)),
        CLEARANCE_P,
        &control_out,
    );
    o.result.as_ref().expect("the hosted control equals the original profile's first hand-off");
    let r = report(&o);
    assert!(line_of(&r, "sh-witness-original").contains("equal=1"), "{r}");
    assert!(line_of(&r, "sh-witness-verdict").contains("paired with the original profile"), "{r}");
    // a hosted first hand-off that differs (another terrain elevation) is a failure naming the field
    let o = clearance(
        &format!("mode = 'clearance_control', original = '{}'", path_text(&original_out)),
        "elevation = 1600.5",
    );
    assert_failed(&o, "FAILURE", "differs from the original profile's: elevation");
    assert!(!report(&o).contains("sh-witness-original"), "{}", report(&o));
    // a run in which the host WAS asked is not the original-solver profile; one with no complete hand-off is UNREACHED
    let o = clearance("mode = 'clearance_original'", "");
    assert_failed(&o, "FAILURE", "this is not the original-solver profile");
    let o = clearance("mode = 'clearance_original'", "original = true, frames = { 'step', 'step' }");
    assert_failed(&o, "UNREACHED", "no complete same-caller hand-off");
    remove(&[&original_out, &control_out]);
}

#[test]
fn an_unpaired_control_is_explicitly_unpaired_and_unlocks_no_intervention() {
    let control_out = temp_path("unpaired");
    let _ = std::fs::remove_file(&control_out);
    let o = run_to(&format!("{CLEARANCE_ARGS}, mode = 'clearance_control'"), CLEARANCE_P, &control_out);
    o.result.as_ref().expect("a standalone control passes as UNPAIRED");
    let r = report(&o);
    assert!(line_of(&r, "sh-witness-verdict").contains("UNPAIRED: no parity claim"), "{r}");
    assert!(!r.contains("sh-witness-original"), "{r}");
    let o = clearance(
        &format!("mode = 'clearance_changed_word', window = 2, control = '{}'", path_text(&control_out)),
        "",
    );
    assert_refused_before_any_effect(&o, "not a PAIRED control");
    remove(&[&control_out]);
}

#[test]
fn a_reference_report_that_is_not_whole_successful_and_of_the_same_setup_is_refused_before_any_side_effect() {
    let p = paired("");
    let original = std::fs::read_to_string(&p.original).unwrap();
    let control = std::fs::read_to_string(&p.control).unwrap();
    // original= (consumed by the control)
    for how in [
        "no_verdict",
        "failed",
        "duplicate_record",
        "other_setup",
        "unknown",
        "fatal",
        "empty_line",
        "wrong_mode",
        "no_counts",
        "schema1",
        "schema2",
        "wrong_profile",
    ] {
        let file = write_temp(&damaged(&original, how));
        let o = clearance(&format!("mode = 'clearance_control', original = '{}'", path_text(&file)), "");
        assert!(o.result.is_err(), "original damaged by {how} must be refused");
        assert!(o.report.is_none() && o.log.is_empty(), "original damaged by {how}: no side effect: {:?}", o.log);
        remove(&[&file]);
    }
    // control= (consumed by an intervention): the same single faults, plus the records an intervention needs
    for how in [
        "no_verdict",
        "failed",
        "duplicate_record",
        "other_setup",
        "unknown",
        "fatal",
        "empty_line",
        "no_counts",
        "schema1",
        "no_trajectory",
        "unpaired",
        "profile_original",
        "schema2",
    ] {
        let file = write_temp(&damaged(&control, how));
        let o = clearance(&format!("mode = 'clearance_changed_word', window = 2, control = '{}'", path_text(&file)), "");
        assert!(o.result.is_err(), "control damaged by {how} must be refused");
        assert!(o.report.is_none() && o.log.is_empty(), "control damaged by {how}: no side effect: {:?}", o.log);
        remove(&[&file]);
    }
    // a control of another window, a missing file, a control for a mode that takes none
    let o = clearance(
        &format!("mode = 'clearance_changed_word', window = 3, control = '{}'", path_text(&p.control)),
        "",
    );
    assert_refused_before_any_effect(&o, "is not the control's window");
    let o = clearance("mode = 'clearance_changed_word', window = 2, control = 'R:/nowhere/control.txt'", "");
    assert_refused_before_any_effect(&o, "cannot read the control report");
    let o = clearance(&format!("mode = 'clearance_control', control = '{}'", path_text(&p.control)), "");
    assert_refused_before_any_effect(&o, "control=<report> is only for a clearance mode that intervenes");
    // the coherent pair still passes (the single faults changed ONE thing each)
    p.intervene("clearance_changed_word", "")
        .result
        .as_ref()
        .expect("the undamaged pair is accepted");
    p.clean();
}

#[test]
fn the_clearance_control_credits_the_first_window_followed_by_a_complete_same_caller_hand_off_and_nothing_else() {
    let o = clearance("mode = 'clearance_control'", "");
    o.result.as_ref().expect("the control with an eligible window and a hand-off passes");
    let r = report(&o);
    // frame 1 is a scan whose FIRST read is not a window; frame 2 is a wrapper-only call (a window with no hand-off); frame 3 hands over
    assert!(
        line_of(&r, "sh-witness-counts").contains("windows=2 windows_without_handoff=1 scan_reads=1 "),
        "{r}"
    );
    let handoff = line_of(&r, "sh-witness-handoff");
    assert!(
        handoff.contains("window=2 reads=2 sector=7,9 real_word=0x00000000 answered_word=0x00000000 "),
        "{handoff}"
    );
    assert!(
        handoff.contains(&format!("elevation={} min_height={} ", bits(1500.5), bits(1500.5))),
        "{handoff}"
    );
    assert!(handoff.contains(&format!("answered_y={}", bits(750.25))), "{handoff}");
    // the trajectory of the hand-off window: window 2 at frame 3 after one window without a hand-off, one scan read and no hand-off before
    assert!(
        line_of(&r, "sh-witness-trajectory").ends_with("window=2 frame=3 windows_without_handoff=1 scan_reads=1 handoffs=0"),
        "{r}"
    );
    assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{r}");
    // the control stops at the first hand-off: the fourth frame is never run
    assert_eq!(o.log.iter().filter(|l| l.as_str() == "move fixed_wing_move").count(), 1, "{:?}", o.log);
}

// the consumed reference's physical words and effective setup (ENV-SECTOR-HEIGHT-WITNESS-CONSUMER-JOIN-1, residual of the review 507f49b7): positive first, then ONE removal or ONE damage at a time
#[test]
fn a_control_reference_must_hold_every_physical_word_finite_and_whole_before_anything_is_compared_or_credited() {
    let p = paired("");
    let original = std::fs::read_to_string(&p.original).unwrap();
    let control = std::fs::read_to_string(&p.control).unwrap();
    // positive first: the coherent pair and a valid changed-word run, whose effect is a stated comparison of two REAL words
    let o = p.intervene("clearance_changed_word", "");
    o.result.as_ref().expect("the valid changed-word run passes");
    let effect = line_of(&report(&o), "sh-witness-effect").to_string();
    assert!(effect.contains("handoff=changed physical=") && !effect.contains("nil"), "{effect}");
    // ONE removal or ONE damage of a field the intervention consumes: refused BEFORE the output, the boot or the intervention, never a changed physical output
    let needle_word = |key: &str| format!("hand-off {key} is absent or not a finite raw word");
    let cases: Vec<(String, String)> = vec![
        ("drop_handoff_answered_y".into(), needle_word("answered_y")),
        ("drop_handoff_min_height".into(), needle_word("min_height")),
        ("drop_handoff_elevation".into(), needle_word("elevation")),
        ("drop_handoff_pre_y".into(), needle_word("pre_y")),
        ("set_handoff_answered_y=nil".into(), needle_word("answered_y")),
        ("set_handoff_min_height=nil".into(), needle_word("min_height")),
        ("set_handoff_answered_y=0x1234".into(), needle_word("answered_y")),
        ("set_handoff_answered_y=0x3F80000G".into(), needle_word("answered_y")),
        ("set_handoff_answered_y=0x7f800000".into(), needle_word("answered_y")),
        ("set_handoff_answered_y=0xff800000".into(), needle_word("answered_y")),
        ("set_handoff_answered_y=0x7fc00000".into(), needle_word("answered_y")),
        ("set_handoff_min_height=0x7f800001".into(), needle_word("min_height")),
        ("set_handoff_min_height=0xffc00000".into(), needle_word("min_height")),
        ("drop_handoff_host_id".into(), "hand-off host_id is absent".into()),
        ("set_handoff_host_id=nil".into(), "hand-off host_id is absent".into()),
        ("drop_handoff_phase".into(), "hand-off phase is absent or malformed".into()),
        ("set_handoff_phase=7".into(), "hand-off phase is absent or malformed".into()),
        ("set_handoff_ordinal=0".into(), "hand-off ordinal is absent or malformed".into()),
        ("drop_handoff_sector".into(), "hand-off sector is absent or malformed".into()),
        (
            "set_handoff_real_word=0x12".into(),
            "hand-off real_word is neither none nor one raw word".into(),
        ),
        ("dup_handoff_answered_y=0x3f800000".into(), "the key answered_y is given twice".into()),
        ("drop_counts_height_reads".into(), "counts height_reads is absent or malformed".into()),
        ("set_counts_altered_reads=1".into(), "altered a window".into()),
        ("set_counts_height_reads=0".into(), "never answered a sector height".into()),
        ("drop_trajectory_frame".into(), "trajectory frame is absent or malformed".into()),
        ("set_trajectory_scan_reads=x".into(), "trajectory scan_reads is absent or malformed".into()),
    ];
    for (how, needle) in &cases {
        let file = write_temp(&damaged(&control, how));
        let o = clearance(&format!("mode = 'clearance_changed_word', window = 2, control = '{}'", path_text(&file)), "");
        let credited = o
            .report
            .as_deref()
            .and_then(|r| r.lines().find(|l| l.starts_with("sh-witness-effect")))
            .unwrap_or("none");
        assert!(o.result.is_err(), "the control damaged by {how} was ACCEPTED and credited ({credited})");
        assert_refused_before_any_effect(&o, needle);
        remove(&[&file]);
    }
    // the same for the first-hand-off reference of the original profile (consumed by the control's parity claim)
    for (how, needle) in [
        ("drop_handoff_answered_y", needle_word("answered_y")),
        ("set_handoff_min_height=nil", needle_word("min_height")),
        ("set_handoff_elevation=0x7f800000", needle_word("elevation")),
        ("drop_handoff_pre_y", needle_word("pre_y")),
        ("set_counts_altered_windows=1", "no first hand-off with the host never asked".to_string()),
    ] {
        let file = write_temp(&damaged(&original, how));
        let o = clearance(&format!("mode = 'clearance_control', original = '{}'", path_text(&file)), "");
        assert_refused_before_any_effect(&o, &needle);
        remove(&[&file]);
    }
    p.clean();
}

#[test]
fn a_reference_must_name_the_same_effective_campaign_and_map_and_never_a_root_path() {
    let p = paired("");
    let original = std::fs::read_to_string(&p.original).unwrap();
    let control = std::fs::read_to_string(&p.control).unwrap();
    // positive first: an explicit override EQUAL to the default is the same effective setup (the old parser accepts this too), then change ONLY the campaign, or ONLY the map, of the run itself: refused before any effect
    let o = clearance(
        &format!(
            r"mode = 'clearance_changed_word', window = 2, campaign = 'LEBANON.CHC', map = '..\\common\\maps\\map5', control = '{}'",
            path_text(&p.control)
        ),
        "",
    );
    o.result
        .as_ref()
        .expect("an explicit override equal to the default is the same effective setup");
    let o = clearance(
        &format!(
            "mode = 'clearance_changed_word', window = 2, campaign = 'GEORGIA.CHC', control = '{}'",
            path_text(&p.control)
        ),
        "",
    );
    assert!(
        o.result.is_err(),
        "a run of ANOTHER effective campaign was accepted against the control ({:?})",
        o.report.as_deref().and_then(|r| r.lines().find(|l| l.starts_with("sh-witness-verdict")))
    );
    assert_refused_before_any_effect(&o, "of another setup: campaign=LEBANON.CHC, this run GEORGIA.CHC");
    let o = clearance(
        &format!(
            r"mode = 'clearance_changed_word', window = 2, map = '..\\common\\maps\\map3', control = '{}'",
            path_text(&p.control)
        ),
        "",
    );
    assert!(
        o.result.is_err(),
        "a run of ANOTHER effective map was accepted against the control ({:?})",
        o.report.as_deref().and_then(|r| r.lines().find(|l| l.starts_with("sh-witness-verdict")))
    );
    assert_refused_before_any_effect(&o, "of another setup: map=");
    // the explicit-default run's own identity names the effective setup
    let o = clearance(
        &format!(
            r"mode = 'clearance_changed_word', window = 2, campaign = 'LEBANON.CHC', map = '..\\common\\maps\\map5', control = '{}'",
            path_text(&p.control)
        ),
        "",
    );
    let line = line_of(&report(&o), "sh-witness-identity").to_string();
    assert!(line.contains(r" campaign=LEBANON.CHC map=..\common\maps\map5 "), "{line}");
    // the references carry the effective setup; two fresh roots are separate paths of ONE setup (the pair above was made from two roots and passes)
    for report in [&original, &control] {
        assert!(
            line_of(report, "sh-witness-identity").contains(r" campaign=LEBANON.CHC map=..\common\maps\map5 "),
            "{report}"
        );
    }
    // change ONLY the effective campaign or ONLY the effective map of the supplied control: refused before any effect
    for (how, needle) in [
        ("set_identity_campaign=GEORGIA.CHC", "of another setup: campaign=GEORGIA.CHC"),
        (r"set_identity_map=..\common\maps\map3", "of another setup: map="),
        ("drop_identity_campaign", "of another setup: campaign=nil"),
        ("drop_identity_map", "of another setup: map=nil"),
    ] {
        let file = write_temp(&damaged(&control, how));
        let o = clearance(&format!("mode = 'clearance_changed_word', window = 2, control = '{}'", path_text(&file)), "");
        assert_refused_before_any_effect(&o, needle);
        remove(&[&file]);
        let file = write_temp(&damaged(&original, how));
        let o = clearance(&format!("mode = 'clearance_control', original = '{}'", path_text(&file)), "");
        assert_refused_before_any_effect(&o, needle);
        remove(&[&file]);
    }
    // an override that is not one token cannot be a setup identity
    let o = clearance(
        &format!(
            "mode = 'clearance_changed_word', window = 2, map = 'a b', control = '{}'",
            path_text(&p.control)
        ),
        "",
    );
    assert_refused_before_any_effect(&o, "map= must be one non-empty token");
    p.clean();
}

#[test]
fn a_save_report_binds_the_map_and_names_a_campaign_but_the_generated_script_of_the_load_is_a_different_campaign() {
    let (path, saved) = healthy_save();
    // the positive relationship stays valid: the save booted the original campaign, the load boots the generated saved script `alpha.Sch`, and the saved raw NaN and infinite histories are untouched by the finiteness rule of the clearance fields
    assert!(
        line_of(&saved, "sh-witness-identity").contains(r" campaign=LEBANON.CHC map=..\common\maps\map5 "),
        "{saved}"
    );
    assert!(saved.contains("word=0x7fc00000") && saved.contains("word=0xff800000"), "{saved}");
    let o = load_run(&path, &saved, &format!("mode = 'load', name = 'beta', seeds = '{SEEDS}'"), "");
    let loaded = o.result.as_ref().map(|_| report(&o)).expect("the load of the generated script passes");
    assert!(line_of(&loaded, "sh-witness-identity").contains(" campaign=alpha.Sch map="), "{loaded}");
    // change ONLY the save report's effective map, or remove its campaign or map: refused before any load side effect
    for (how, needle) in [
        (r"set_identity_map=..\common\maps\map3", "of another setup: map="),
        ("drop_identity_map", "of another setup: map=nil"),
        ("drop_identity_campaign", "names no effective campaign"),
        ("set_identity_campaign=none", "names no effective campaign"),
    ] {
        let file = write_temp(&damaged(&saved, how));
        let o = load_run(&file, &saved, "mode = 'load', name = 'beta'", "");
        assert_refused_before_any_effect(&o, needle);
        remove(&[&file]);
    }
    remove(&[&path]);
}

#[test]
fn a_clearance_control_with_no_hand_off_is_unreached_never_a_pass_and_scan_reads_are_never_windows() {
    let o = clearance("mode = 'clearance_control'", "frames = { 'scan', 'step', 'step' }");
    assert_failed(&o, "UNREACHED", "no complete same-caller hand-off");
    assert!(
        line_of(&report(&o), "sh-witness-counts").contains("windows=2 windows_without_handoff=2 scan_reads=1"),
        "{}",
        report(&o)
    );
    let o = clearance("mode = 'clearance_control'", "frames = { 'scan', 'scan' }");
    assert_failed(&o, "UNREACHED", "no complete same-caller hand-off");
    assert!(
        line_of(&report(&o), "sh-witness-counts").contains("windows=0 windows_without_handoff=0 scan_reads=2"),
        "{}",
        report(&o)
    );
}

#[test]
fn another_host_callback_between_two_reads_ends_a_window() {
    let o = clearance("mode = 'clearance_control'", "frames = { 'split', 'handoff' }, window_of = { [2] = 3 }");
    o.result.as_ref().expect("the second frame hands over");
    // the split frame is two windows of one read each, the hand-off frame a third
    assert!(
        line_of(&report(&o), "sh-witness-counts").contains("windows=3 windows_without_handoff=2"),
        "{}",
        report(&o)
    );
    assert!(line_of(&report(&o), "sh-witness-handoff").contains("window=3 reads=2"), "{}", report(&o));
}

#[test]
fn a_hand_off_is_credited_only_for_a_complete_same_caller_sequence_with_the_hosts_real_positive_answers() {
    // from the coherent positive (the control above), change ONE thing between the reads and the move
    for (extra, why) in [
        ("lookahead_id = 'ac2'", "the look-ahead of another caller than the move"),
        ("move_id = 'ac2'", "the move of another caller than the look-ahead"),
        ("between = 'bind'", "a bind between the look-ahead and the move"),
        ("between = 'model'", "a model question between the look-ahead and the move"),
        ("between = 'move'", "another vehicle's step between the look-ahead and the move"),
    ] {
        let o = clearance("mode = 'clearance_control'", extra);
        assert_failed(&o, "UNREACHED", "no complete same-caller hand-off");
        assert!(!report(&o).contains("sh-witness-handoff"), "{why}: {}", report(&o));
    }
    // incomplete current evidence, or a host answer without its position or without a look-ahead answer, is a failure and never a hand-off
    for (extra, needle) in [
        ("missing = 'phase'", "no phase"),
        ("missing = 'elevation'", "no finite elevation"),
        ("missing = 'pose'", "no finite pre pose position"),
        ("answer_missing = true", "the host's answer holds no finite position"),
        ("lookahead_nil = true", "the host answered no table"),
    ] {
        let o = clearance("mode = 'clearance_control'", extra);
        let error = o.result.as_ref().expect_err(extra);
        assert!(error.contains("FAILURE") && error.contains(needle), "{extra}: {error}");
        assert!(!report(&o).contains("sh-witness-handoff"), "{extra}");
    }
    // a missing host id in the move can never be the same caller as the look-ahead
    let o = clearance("mode = 'clearance_control'", "missing = 'id'");
    assert_failed(&o, "UNREACHED", "no complete same-caller hand-off");
}

#[test]
fn a_changed_host_word_changes_the_campaigns_hand_off_and_the_hosts_physical_answer() {
    let p = paired("");
    let o = p.intervene("clearance_changed_word", "");
    o.result.as_ref().expect("the changed word reaches the hand-off");
    let r = report(&o);
    let handoff = line_of(&r, "sh-witness-handoff");
    // 100000.0 - 1500.5 = 98499.5 exactly in single precision; the host's answered height is half of it
    assert!(
        handoff.contains(&format!(
            "answered_word=0x47c35000 host_id=ac1 elevation={} min_height={} ",
            bits(1500.5),
            bits(98499.5)
        )),
        "{handoff}"
    );
    assert!(handoff.contains(&format!("answered_y={}", bits(49249.75))), "{handoff}");
    assert!(line_of(&r, "sh-witness-effect").contains("handoff=changed physical=changed"), "{r}");
    assert!(line_of(&r, "sh-witness-counts").contains("altered_windows=1 altered_reads=2"), "{r}");
    // a host that answers the same height whatever it is handed: the hand-off effect is stated, the physical effect is stated UNCHANGED, never claimed
    let o = p.intervene("clearance_changed_word", "host_y_const = 750.25");
    o.result.as_ref().expect("the hand-off still follows the word");
    assert!(
        line_of(&report(&o), "sh-witness-effect").contains("handoff=changed physical=unchanged"),
        "{}",
        report(&o)
    );
    // a hand-off that does not follow the changed word (the campaign ignores it) is a hand-off that matches neither the word's value nor the control's: FAILURE, never a pass
    let o = p.intervene("clearance_changed_word", "ignores_word = true");
    assert_failed(&o, "FAILURE", "neither the round-toward-zero value");
    // a hand-off one step above the accepted value is wrong
    let o = p.intervene("clearance_changed_word", "bump = 0.0078125");
    assert_failed(&o, "FAILURE", "neither the round-toward-zero value");
    // the consumed hand-off of ANOTHER caller or step than the control's is not the control's
    let o = p.intervene("clearance_changed_word", "caller = 'ac9'");
    assert_failed(&o, "FAILURE", "not the control's caller and step (host_id");
    let o = p.intervene("clearance_changed_word", "elevation = 1600.5");
    assert_failed(&o, "FAILURE", "not the control's caller and step (elevation");
    // no complete hand-off in the altered window is UNREACHED, with the altered read not credited as consumed
    let o = p.intervene("clearance_changed_word", "move_id = 'ac2'");
    assert_failed(&o, "UNREACHED", "no complete same-caller hand-off in window 2");
    // a window never reached
    let o = p.intervene("clearance_changed_word", "frames = { 'scan', 'step' }");
    assert_failed(&o, "UNREACHED", "no complete same-caller hand-off in window 2");
    // a run whose trajectory is not the control's is never intervened on
    let o = p.intervene("clearance_changed_word", "frames = { 'step', 'step', 'handoff' }");
    assert_failed(&o, "FAILURE", "the trajectory of window 2 differs from the control's");
    assert!(
        line_of(&report(&o), "sh-witness-counts").contains("altered_windows=0 altered_reads=0"),
        "{}",
        report(&o)
    );
    p.clean();
}

#[test]
fn a_clean_non_dominance_is_unreached_and_an_unexplained_hand_off_is_a_failure() {
    // an altitude term (200000.0) dominates the clearance expression in the control and in the changed run: the hand-off equals the control's and is larger than the word's value
    let p = paired("wp_alt = 200000.0");
    let o = p.intervene("clearance_changed_word", "wp_alt = 200000.0");
    assert_failed(&o, "UNREACHED", "did not dominate the clearance expression");
    p.clean();
}

#[test]
fn the_expected_clearance_subtraction_uses_the_accepted_round_toward_zero_profile_not_round_to_nearest() {
    // a finite NON-EXACT subtraction where round-to-nearest and round-toward-zero differ, computed by the scripted campaign under the profile the driver ACTUALLY selected (the harness reads it back and restores it)
    let elevation = elevation_where_nearest_and_toward_zero_differ(100000.0, 1500.1);
    let exact = f64::from(100000.0f32) - f64::from(elevation);
    let down = truncate_to_single(exact);
    assert!(
        f64::from(down) != exact && f64::from(f32::from_bits(down.to_bits() + 1)) - exact < exact - f64::from(down),
        "the precondition: nearest and toward-zero differ"
    );
    let extra = format!("elevation = {:?}", f64::from(elevation));
    let p = paired(&extra);
    let o = p.intervene("clearance_changed_word", &extra);
    o.result.as_ref().expect("the accepted toward-zero hand-off is accepted");
    let r = report(&o);
    assert!(
        line_of(&r, "sh-witness-handoff").contains(&format!("min_height={} ", bits(down))),
        "the hand-off is the toward-zero value: {r}"
    );
    // the genuinely wrong hand-off (the round-to-nearest value, one unit above) is rejected
    let o = p.intervene(
        "clearance_changed_word",
        &format!("{extra}, bump = {:?}", f64::from(f32::from_bits(down.to_bits() + 1)) - f64::from(down)),
    );
    assert_failed(&o, "FAILURE", "neither the round-toward-zero value");
    p.clean();
    // the control's lower bound is the same expression: a held word whose difference is non-exact, the hand-off the toward-zero value, accepted
    let word = 6000.0f32;
    let elevation = elevation_where_nearest_and_toward_zero_differ(word, 1500.1);
    let extra = format!(
        "elevation = {:?}, held = {{ [BITS (7) .. ',' .. BITS (9)] = '{}' }}",
        f64::from(elevation),
        bits(word)
    );
    let o = clearance("mode = 'clearance_control'", &extra);
    o.result.as_ref().expect("the control's lower bound is formed under the accepted profile");
    let handoff = report(&o);
    let expected = truncate_to_single(f64::from(word) - f64::from(elevation));
    assert!(
        line_of(&handoff, "sh-witness-handoff").contains(&format!("min_height={} ", bits(expected))),
        "{handoff}"
    );
    // an exact positive stays accepted (the 98499.5 case above) and a hand-off BELOW the lower bound is rejected
    let o = clearance("mode = 'clearance_control'", &format!("{extra}, bump = -1.0"));
    assert_failed(&o, "FAILURE", "is below max(elevation, word - elevation)");
    // a subtraction the double cannot hold exactly is refused as unformable, never rounded twice
    let o = clearance("mode = 'clearance_control'", "held = { [BITS (7) .. ',' .. BITS (9)] = '0x7e800000' }");
    assert_failed(&o, "FAILURE", "cannot be formed exactly in single precision");
}

#[test]
fn each_refusal_mode_is_credited_only_when_the_campaign_stops_at_its_marker_after_the_eligible_read() {
    let p = paired("");
    for (mode, marker) in [
        ("clearance_withheld", "entity_scan.callback"),
        ("clearance_malformed", "entity_scan.answer"),
        ("clearance_wrong_state", "entity_scan.state"),
        ("clearance_wrong_sector", "entity_scan.sector_height"),
    ] {
        // positive first: the paired control of the same campaign is healthy; the single fault stops at its marker
        let o = p.intervene(mode, "");
        o.result.as_ref().unwrap_or_else(|e| panic!("{mode}: {e}"));
        let r = report(&o);
        assert!(line_of(&r, "sh-witness-fatal").contains(&format!("marker={marker}")), "{mode}: {r}");
        assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{mode}: {r}");
        // a fatal BEFORE the eligible read, even at the same marker, is a failure
        let o = p.intervene(mode, &format!("break_at_window = 1, break_marker = '{marker}'"));
        assert_failed(&o, "FAILURE", "before the eligible read of window 2");
        // a campaign that accepts the faulty answer and goes on is a failure (a withheld answer is a host that raised: nothing can accept it)
        if mode != "clearance_withheld" {
            let o = p.intervene(mode, "lenient = true");
            assert_failed(&o, "FAILURE", "accepted the faulty answer");
        }
        // a window never reached is UNREACHED
        let o = p.intervene(mode, "frames = { 'scan', 'step' }");
        assert_failed(&o, "UNREACHED", "window 2 was never reached");
        // a run whose trajectory is not the control's is never intervened on
        let o = p.intervene(mode, "frames = { 'step', 'step', 'handoff' }");
        assert_failed(&o, "FAILURE", "the trajectory of window 2 differs from the control's");
        assert!(!report(&o).contains("sh-witness-fatal"), "{mode}: no fault was delivered: {}", report(&o));
        // a malformed positive reply (a trailing comma) or request (an identity, an extra comma, a non-zero fact) at the eligible read is the host's or the campaign's own failure, never the configured single fault
        for extra in [
            "reply = 'trailing', reply_nth = 3",
            "reply = 'empty', reply_nth = 3",
            "reply = 'name', reply_nth = 3",
            "reply = 'extra', reply_nth = 3",
            "reply = 'result', reply_nth = 3",
        ] {
            let o = p.intervene(mode, extra);
            assert_failed(&o, "FAILURE", "the host's own answer broke the contract");
            assert!(
                !report(&o).contains("after the one intervention"),
                "{mode} {extra}: the configured fault is not credited: {}",
                report(&o)
            );
        }
        for extra in [
            "bad_request = 'identity', bad_request_nth = 3",
            "bad_request = 'comma', bad_request_nth = 3",
            "bad_request = 'nonzero', bad_request_nth = 3",
        ] {
            let o = p.intervene(mode, extra);
            assert_failed(&o, "FAILURE", "the campaign's own request broke the contract");
        }
    }
    // the right marker is required: a campaign whose bridge names another marker for the withheld answer is a failure
    let o = p.intervene("clearance_withheld", "callback_marker = 'entity_scan.refused'");
    assert_failed(&o, "FAILURE", "not at [model-provider:entity_scan.callback]");
    p.clean();
}

#[test]
fn a_fatal_or_a_request_or_reply_fault_outranks_a_clean_unreached() {
    // no hand-off in the run, and a host answer that breaks the grammar (or a fatal): FAILURE, never the clean UNREACHED the absence alone would be
    let o = clearance("mode = 'clearance_control'", "frames = { 'step', 'step' }, reply = 'trailing', reply_nth = 1");
    assert_failed(&o, "FAILURE", "the host's own answer broke the contract");
    let o = clearance("mode = 'clearance_control'", "frames = { 'step', 'step' }, bad_request = 'identity'");
    assert_failed(&o, "FAILURE", "the campaign's own request broke the contract");
    let o = clearance("mode = 'clearance_control'", "frames = { 'step', 'step' }, break_at_window = 1");
    assert_failed(&o, "FAILURE", "the engine stopped");
    let o = clearance(
        "mode = 'clearance_original'",
        "original = true, frames = { 'step', 'step' }, break_at_window = 1",
    );
    assert_failed(&o, "FAILURE", "the engine stopped");
}

#[test]
fn the_clearance_modes_check_their_arguments_before_any_side_effect() {
    let o = clearance("mode = 'clearance_changed_word'", "");
    assert!(o.result.as_ref().expect_err("a missing window").contains("needs window=<N>"));
    assert!(o.report.is_none() && o.log.is_empty(), "{:?}", o.log);
    let o = clearance("mode = 'clearance_changed_word', window = 2", "");
    assert_refused_before_any_effect(&o, "needs control=<the successful paired clearance_control report>");
    let o = clearance("mode = 'clearance_nowhere'", "");
    assert!(o.result.as_ref().expect_err("an unknown mode").contains("is not a mode of this witness"));
    let o = clearance("mode = 'clearance_control', window = 2", "");
    assert!(o
        .result
        .as_ref()
        .expect_err("a window for the control")
        .contains("window= is only for a clearance mode that intervenes"));
    let o = clearance("mode = 'clearance_control', seeds = '1:1:0x3f800000'", "");
    assert!(o
        .result
        .as_ref()
        .expect_err("seeds for a clearance mode")
        .contains("seeds= is only for a session mode"));
    let o = clearance("mode = 'clearance_control', expect = 'x'", "");
    assert_refused_before_any_effect(&o, "expect=<report> is only for a load mode");
    let o = clearance("mode = 'clearance_control', scenario = 'nowhere'", "");
    assert_refused_before_any_effect(&o, "scenario=");
    // an existing report is never overwritten and nothing boots
    let existing = temp_path("existing");
    std::fs::write(&existing, "earlier evidence\n").unwrap();
    let o = run_to(&format!("{CLEARANCE_ARGS}, mode = 'clearance_control'"), CLEARANCE_P, &existing);
    assert!(o.result.as_ref().expect_err("an existing report").contains("cannot create"));
    assert_eq!(std::fs::read_to_string(&existing).unwrap(), "earlier evidence\n");
    assert!(o.log.is_empty(), "{:?}", o.log);
    remove(&[&existing]);
}

// ---------------------------------------------------------------------------------------------------------------- M6: the native save and the campaign's own load

const SESSION_P: &str = "map = (function () local t = {} for x = 0, 3 do for z = 0, 2 do t [#t + 1] = { BITS (x), BITS (z) } end end return t end) ()";
const SEEDS: &str = "1:1:0xc0a00000,2:1:0x7fc00000,3:2:0xff800000";

fn save_run(extra_args: &str, extra_p: &str, family: &str) -> Outcome {
    run(
        &format!("root = 'x', out = '@OUT@', {extra_args}"),
        &format!("family = '{family}', {SESSION_P}, {extra_p}"),
    )
}

/// a save run that passes, its report kept for the load that follows
fn healthy_save() -> (std::path::PathBuf, String) {
    let out = temp_path("save");
    let _ = std::fs::remove_file(&out);
    let o = run_to(
        &format!("root = 'x', out = '@OUT@', mode = 'save', name = 'alpha', seeds = '{SEEDS}'"),
        &format!("family = 'save', {SESSION_P}"),
        &out,
    );
    o.result.as_ref().expect("the healthy save passes");
    let report = report(&o);
    (out, report)
}

fn saved_words_table(report: &str) -> String {
    let mut entries = Vec::new();
    for line in report.lines().filter(|l| l.starts_with("sh-save-word ")) {
        let field = |key: &str| line.split_whitespace().find_map(|t| t.strip_prefix(key)).unwrap().to_string();
        let (x, z, word) = (field("x=").parse::<f32>().unwrap(), field("z=").parse::<f32>().unwrap(), field("word="));
        entries.push(format!("[ '{},{}' ] = '{word}'", bits(x), bits(z)));
    }
    format!("saved_words = {{ {} }}", entries.join(", "))
}

#[test]
fn the_native_save_records_every_packed_word_and_every_seeded_sector_with_its_seeds_exact_bits() {
    let (path, r) = healthy_save();
    assert!(
        line_of(&r, "sh-witness-counts").contains("pack_reads=12 pack_other_questions=0 restores=0 "),
        "{r}"
    );
    assert!(line_of(&r, "sh-witness-counts").contains("seeds_checked=3"), "{r}");
    assert!(
        line_of(&r, "sh-witness-pack").contains("attempts=1 unique=12 repeat_reads=0 consistent=1"),
        "{r}"
    );
    assert_eq!(r.lines().filter(|l| l.starts_with("sh-save-word ")).count(), 12);
    assert!(
        r.contains("x=1 z=1 word=0xc0a00000") && r.contains("x=2 z=1 word=0x7fc00000") && r.contains("x=3 z=2 word=0xff800000"),
        "{r}"
    );
    assert!(r.contains("sh-save-seed sx=1 sz=1 word=0xc0a00000"), "{r}");
    assert!(
        line_of(&r, "sh-witness-save name=alpha bytes=4096 version=7").starts_with("sh-witness-save"),
        "{r}"
    );
    assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{r}");
    remove(&[&path]);
}

#[test]
fn the_real_stores_pack_retry_is_reconciled_and_every_inconsistent_or_unsupported_repetition_is_a_failure() {
    let base = format!("mode = 'save', name = 'alpha', seeds = '{SEEDS}'");
    // a coherent real-writer-order overflow and retry, then the successful save: every attempt reads every sector again in the same order with the same words
    let o = save_run(&base, "pack_attempts = 3", "save");
    o.result.as_ref().expect("a legitimate retry passes");
    let r = report(&o);
    assert!(
        line_of(&r, "sh-witness-pack").contains("attempts=3 unique=12 repeat_reads=24 consistent=1"),
        "{r}"
    );
    assert_eq!(
        r.lines().filter(|l| l.starts_with("sh-save-word ")).count(),
        12,
        "the unique final relationship, not the repeats: {r}"
    );
    assert!(line_of(&r, "sh-witness-verdict").contains("3 attempt(s)"), "{r}");
    // damage ONE repeated raw word, omit one sector of a repeat, a first attempt that stopped short, a sector read twice inside one attempt: each is non-pass
    let o = save_run(
        &base,
        "pack_attempts = 2, pack_damage = { attempt = 2, key = BITS (0) .. ',' .. BITS (0), word = '0x3f800000' }",
        "save",
    );
    assert_failed(&o, "FAILURE", "an inconsistent repeated word");
    let o = save_run(&base, "pack_attempts = 2, pack_omit = 2", "save");
    assert_failed(&o, "FAILURE", "unsupported repetition");
    let o = save_run(&base, "pack_attempts = 2, pack_short_first = true", "save");
    assert_failed(&o, "FAILURE", "unsupported repetition");
    let o = save_run(&base, "pack_twice = true", "save");
    assert_failed(&o, "FAILURE", "unsupported repetition");
    let o = save_run(&base, "pack_attempts = 65", "save");
    assert_failed(&o, "FAILURE", "more than the doubling");
    // the same in the load's second pack: a retry passes, a damaged repeat or an omitted sector fails
    let (path, saved) = healthy_save();
    let o = load_run(&path, &saved, "mode = 'load', name = 'beta'", "pack_attempts = 2");
    o.result.as_ref().expect("a retry in the second pack of a load passes");
    assert!(
        line_of(&report(&o), "sh-witness-pack").contains("attempts=2 unique=12 repeat_reads=12 consistent=1"),
        "{}",
        report(&o)
    );
    let o = load_run(
        &path,
        &saved,
        "mode = 'load', name = 'beta'",
        "pack_attempts = 2, pack_damage = { attempt = 2, key = BITS (3) .. ',' .. BITS (0), word = '0x3f800000' }",
    );
    assert_failed(&o, "FAILURE", "an inconsistent repeated word");
    let o = load_run(&path, &saved, "mode = 'load', name = 'beta'", "pack_attempts = 2, pack_omit = 2");
    assert_failed(&o, "FAILURE", "unsupported repetition");
    remove(&[&path]);
}

#[test]
fn every_single_fault_of_the_save_is_a_failure_or_a_refusal_never_a_pass() {
    let base = "mode = 'save', name = 'alpha'";
    // a seeded sector the session never packs
    let o = save_run(&format!("{base}, seeds = '9:9:0x3f800000'"), "", "save");
    assert_failed(&o, "FAILURE", "was never packed");
    // a save report that is not whole
    let o = save_run(
        &format!("{base}, seeds = '{SEEDS}'"),
        "save_line = 'sh-witness-save name=%s bytes=2 version=7\\n'",
        "save",
    );
    assert_failed(&o, "FAILURE", "not whole");
    // the module has no witness method (built without the feature)
    let o = save_run(&format!("{base}, seeds = '{SEEDS}'"), "no_save_method = true", "save");
    assert_failed(&o, "FAILURE", "without feature eech-dc/sector-height-witness");
    // the engine refuses (a saved file of the name exists): the refusal is the verdict, nothing is credited
    let o = save_run(&format!("{base}, seeds = '{SEEDS}'"), "save_refused = true", "save");
    assert_failed(&o, "FAILURE", "a saved file of that name exists");
    // a host that does not hold or acknowledge a seed, or answers its read malformed: refused before the save, never credited
    let o = save_run(&format!("{base}, seeds = '{SEEDS}'"), "stored_wrong = BITS (1) .. ',' .. BITS (1)", "save");
    assert_failed(&o, "FAILURE", "did not acknowledge the seed");
    let o = save_run(&format!("{base}, seeds = '{SEEDS}'"), "seed_reply = 'trailing'", "save");
    assert_failed(&o, "FAILURE", "did not acknowledge the seed");
    assert!(!o.log.iter().any(|l| l.starts_with("save ")), "{:?}", o.log);
    // a pack read the host answers malformed is the host's own failure
    let o = save_run(
        &format!("{base}, seeds = '{SEEDS}'"),
        "leader_sector = { BITS (1), BITS (1) }, reply = 'trailing', reply_nth = 2",
        "save",
    );
    assert_failed(&o, "FAILURE", "the host's own answer broke the contract");
    let o = save_run(
        &format!("{base}, seeds = '{SEEDS}'"),
        "leader_sector = { BITS (1), BITS (1) }, bad_request = 'identity'",
        "save",
    );
    assert_failed(&o, "FAILURE", "the campaign's own request broke the contract");
    // refused before any side effect: a signalling NaN is unmeasured, a duplicate, malformed, empty-segment or trailing-comma seed, a bad name
    for (args, needle) in [
        (format!("{base}, seeds = '1:1:0x7f800001'"), "signalling NaN"),
        (format!("{base}, seeds = '1:1:0xffc00001,1:1:0x3f800000'"), "seeded twice"),
        (format!("{base}, seeds = '1:1:3f800000'"), "sx:sz:0xWORD"),
        (format!("{base}, seeds = '1:1:0x3f800000,,2:2:0x3f800000'"), "sx:sz:0xWORD"),
        (format!("{base}, seeds = '1:1:0x3f800000,'"), "end with a comma"),
        (format!("{base}, seeds = ',1:1:0x3f800000'"), "sx:sz:0xWORD"),
        ("mode = 'save', name = '../x'".to_string(), "name=<1 to 24"),
        ("mode = 'save', name = 'aaaaaaaaaaaaaaaaaaaaaaaaa'".to_string(), "name=<1 to 24"),
        ("mode = 'save'".to_string(), "name=<1 to 24"),
    ] {
        let o = save_run(&args, "", "save");
        assert_refused_before_any_effect(&o, needle);
    }
    // a quiet NaN, an infinity and a negative word are valid raw history
    let o = save_run(&format!("{base}, seeds = '1:1:0x7fc00001,2:1:0x7f800000,3:2:0x80000000'"), "", "save");
    o.result.as_ref().expect("quiet NaN payloads, infinities and negative zero are valid history");
}

#[test]
fn the_save_withheld_mode_is_credited_only_at_the_first_pack_read_and_only_with_the_marker() {
    let o = save_run("mode = 'save_withheld', name = 'alpha', seeds = '1:1:0xc0a00000'", "", "save");
    o.result.as_ref().expect("the withheld pack read stops the campaign at its marker");
    let r = report(&o);
    assert!(line_of(&r, "sh-witness-fatal").contains("where=save marker=entity_scan.callback"), "{r}");
    let o = save_run("mode = 'save_withheld', name = 'alpha'", "save_refused = true", "save");
    assert_failed(&o, "FAILURE", "before the eligible pack");
}

fn load_p(saved: &str, extra: &str) -> String {
    // inserts: (1,1) holds a negative history (raised by a larger sum), (2,1) a quiet NaN (never changed), (3,2) minus infinity (raised), (0,0) the zero history (raised)
    format!(
        "{saved}, inserts = {{ {{ BITS (1), BITS (1), '0x40a00000' }}, {{ BITS (2), BITS (1), '0x40a00000' }}, {{ BITS (3), BITS (2), '0x40a00000' }}, {{ BITS (0), BITS (0), '0x40400000' }} }}, {extra}"
    )
}

fn load_run(save_report_path: &std::path::Path, report_text: &str, mode_args: &str, extra_p: &str) -> Outcome {
    run(
        &format!(
            "root = 'x', out = '@OUT@', {mode_args}, campaign = 'alpha.Sch', expect = '{}'",
            path_text(save_report_path)
        ),
        &format!("family = 'load', {SESSION_P}, {}", load_p(&saved_words_table(report_text), extra_p)),
    )
}

// the supported boot builds a baseline BEFORE the saved state (a generated script holds CAMPAIGN_DATA before SAVED_CAMPAIGN: static population, parsgen.c), so insertions before the first restore are legitimate telemetry, not a defect and not saved-state reconstruction
const BASELINE: &str = "pre_inserts = { { BITS (1), BITS (1), '0x40000000' }, { BITS (4), BITS (0), '0x40000000' }, { BITS (5), BITS (0), '0x41200000' } }";

#[test]
fn a_legitimate_baseline_population_before_the_first_restore_is_reported_honestly_and_every_saved_state_check_still_holds() {
    let (path, saved) = healthy_save();
    let base = "mode = 'load', name = 'beta'";
    // positive first, with NO baseline (the previous driver also passes this): the coherent load
    load_run(&path, &saved, base, "").result.as_ref().expect("the load without a baseline passes");
    // then three baseline insertions before the first restore, the 12 exact restores, the four saved entity arrivals and the real second pack
    let o = load_run(&path, &saved, base, BASELINE);
    assert!(
        o.result.is_ok(),
        "a legitimate baseline before the first restore was refused as a defect: {:?}",
        o.result
    );
    o.result.as_ref().expect("the load with a legitimate baseline passes");
    let r = report(&o);
    assert!(
        line_of(&r, "sh-witness-counts")
            .contains("restores=12 restore_duplicates=0 inserts=7 inserts_before_first_restore=3 inserts_between_restores=0 inserts_after_last_restore=4"),
        "the baseline telemetry is reported, never zeroed or hidden: {r}"
    );
    let verdict = line_of(&r, "sh-witness-verdict");
    assert!(
        verdict.contains("status=PASS")
            && verdict.contains("3 baseline boot insertions before the first restore (telemetry)")
            && verdict.contains("4 saved-entity insertions after the last restore"),
        "{verdict}"
    );
    // the restored words REPLACE what the baseline held (the baseline raised (1,1) to 2.0; the saved history is the negative word), and the later arrivals raise them as before
    assert!(
        r.contains("x=1 z=1 stated=0xc0a00000 stored=0xc0a00000 final=0x40a00000 relation=raised"),
        "{r}"
    );
    assert!(r.contains("x=2 z=1 stated=0x7fc00000 stored=0x7fc00000 final=0x7fc00000 relation=held"), "{r}");
    // single faults, each WITH the baseline: the saved-state checks are as strict as without it
    let key_11 = "BITS (1) .. ',' .. BITS (1)";
    let with = |extra: &str| format!("{BASELINE}, {extra}");
    let o = load_run(&path, &saved, base, &with(&format!("restore_override = {{ {key_11}, '0xc0a00001' }}")));
    assert_failed(&o, "FAILURE", "restored as 0xc0a00001, the save packed 0xc0a00000");
    let o = load_run(&path, &saved, base, &with("skip_restore = 5"));
    assert_failed(&o, "FAILURE", "the load restored 11 sectors, the save packed 12");
    let o = load_run(&path, &saved, base, &with("dup_restore = 5"));
    assert_failed(&o, "FAILURE", "restored more than once");
    let o = load_run(&path, &saved, base, &with("insert_between = 6"));
    assert_failed(&o, "FAILURE", "between two restores");
    let o = load_run(&path, &saved, base, &with("no_inserts = true"));
    assert_failed(&o, "UNREACHED", "no entity insertion followed");
    let o = load_run(&path, &saved, base, &with(&format!("lowering = {key_11}")));
    assert_failed(&o, "FAILURE", "LOWERED or a NaN changed");
    let o = load_run(&path, &saved, base, &with("no_restore = true"));
    assert_failed(&o, "UNREACHED", "made no restore");
    // a baseline alone is not a load: with no restore and no saved entity arriving the telemetry says so and nothing is credited
    remove(&[&path]);
}

#[test]
fn the_campaigns_own_load_restores_every_saved_word_exactly_and_none_is_lowered_after_the_saved_entities_arrive() {
    let (path, saved) = healthy_save();
    let o = load_run(&path, &saved, "mode = 'load', name = 'beta'", "");
    o.result.as_ref().expect("the healthy load passes");
    let r = report(&o);
    assert!(
        line_of(&r, "sh-witness-counts")
            .contains("restores=12 restore_duplicates=0 inserts=4 inserts_before_first_restore=0 inserts_between_restores=0 inserts_after_last_restore=4"),
        "{r}"
    );
    // the three raw histories came back bit for bit and were then raised only where the arrival is greater: a negative and minus infinity raised, a quiet NaN held
    assert!(
        r.contains("x=1 z=1 stated=0xc0a00000 stored=0xc0a00000 final=0x40a00000 relation=raised"),
        "{r}"
    );
    assert!(r.contains("x=2 z=1 stated=0x7fc00000 stored=0x7fc00000 final=0x7fc00000 relation=held"), "{r}");
    assert!(
        r.contains("x=3 z=2 stated=0xff800000 stored=0xff800000 final=0x40a00000 relation=raised"),
        "{r}"
    );
    assert!(
        r.contains("x=0 z=0 stated=0x00000000 stored=0x00000000 final=0x40400000 relation=raised"),
        "{r}"
    );
    assert!(r.contains("x=3 z=0 stated=0x00000000 stored=0x00000000 final=0x00000000 relation=held"), "{r}");
    assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{r}");
    assert!(
        o.log.iter().any(|l| l == "boot alpha.Sch") && o.log.iter().any(|l| l == "save beta"),
        "{:?}",
        o.log
    );
    remove(&[&path]);
}

#[test]
fn a_save_report_that_is_not_whole_successful_and_matched_is_refused_before_any_load_side_effect() {
    let (path, saved) = healthy_save();
    for how in [
        "no_verdict",
        "failed",
        "duplicate_record",
        "duplicate_row",
        "other_setup",
        "unknown",
        "fatal",
        "empty_line",
        "wrong_mode",
        "no_counts",
        "schema1",
        "no_pack",
        "gap",
        "bad_word",
        "wrong_profile",
        "schema2",
    ] {
        let file = write_temp(&damaged(&saved, how));
        let o = load_run(&file, &saved, "mode = 'load', name = 'beta'", "");
        assert!(o.result.is_err(), "the save report damaged by {how} must be refused");
        assert!(
            o.report.is_none() && o.log.is_empty(),
            "the save report damaged by {how}: no side effect: {:?}",
            o.log
        );
        remove(&[&file]);
    }
    // a saved script of another name, a new name equal to the saved one, seeds that are not the save report's
    let o = run(
        &format!(
            "root = 'x', out = '@OUT@', mode = 'load', name = 'beta', campaign = 'zzz.Sch', expect = '{}'",
            path_text(&path)
        ),
        &format!("family = 'load', {SESSION_P}, {}", load_p(&saved_words_table(&saved), "")),
    );
    assert_refused_before_any_effect(&o, "is not the saved script of the save report's name");
    let o = load_run(&path, &saved, "mode = 'load', name = 'alpha'", "");
    assert_refused_before_any_effect(&o, "must be a NEW saved name");
    let o = load_run(&path, &saved, "mode = 'load', name = 'beta', seeds = '1:1:0x3f800000'", "");
    assert_refused_before_any_effect(&o, "is not the save report's");
    let o = load_run(&path, &saved, &format!("mode = 'load', name = 'beta', seeds = '{SEEDS},0:0:0x3f800000'"), "");
    assert_refused_before_any_effect(&o, "is not the save report's");
    let o = load_run(&path, &saved, "mode = 'load', name = 'beta', seeds = '1:1:0xc0a00000'", "");
    assert_refused_before_any_effect(&o, "not among this run's seeds");
    // the undamaged report with matching seeds still loads
    let o = load_run(&path, &saved, &format!("mode = 'load', name = 'beta', seeds = '{SEEDS}'"), "");
    o.result.as_ref().expect("matching seeds are accepted");
    remove(&[&path]);
}

#[test]
fn every_single_fault_of_the_load_is_a_failure_or_unreached_never_a_pass() {
    let (path, saved) = healthy_save();
    let base = "mode = 'load', name = 'beta'";
    let key_11 = "BITS (1) .. ',' .. BITS (1)";
    // a restore that states another word than the save packed
    let o = load_run(&path, &saved, base, &format!("restore_override = {{ {key_11}, '0xc0a00001' }}"));
    assert_failed(&o, "FAILURE", "restored as 0xc0a00001, the save packed 0xc0a00000");
    // a sector packed but never restored (the count differs first)
    let o = load_run(&path, &saved, base, "skip_restore = 5");
    assert_failed(&o, "FAILURE", "the load restored 11 sectors, the save packed 12");
    // a sector restored twice
    let o = load_run(&path, &saved, base, "dup_restore = 5");
    assert_failed(&o, "FAILURE", "restored more than once");
    // an insertion before the first restore is the supported boot's baseline (a legitimate pass, see the baseline test); one BETWEEN two restores has no legitimate source
    let o = load_run(&path, &saved, base, "insert_first = true");
    o.result
        .as_ref()
        .expect("an insertion before the first restore is legitimate baseline telemetry");
    let o = load_run(&path, &saved, base, "insert_between = 6");
    assert_failed(&o, "FAILURE", "between two restores");
    // no restore at all (the saved script was not loaded as a saved campaign), and restores with no insertion after them
    let o = load_run(&path, &saved, base, "no_restore = true");
    assert_failed(&o, "UNREACHED", "made no restore");
    let o = load_run(&path, &saved, base, "no_inserts = true");
    assert_failed(&o, "UNREACHED", "no entity insertion followed");
    // the host stores bits other than the ones stated: the host's own answer is the failure, whether the campaign's own check also stops the boot (a fatal is recorded) or a lenient campaign goes on
    let o = load_run(&path, &saved, base, &format!("stored_wrong = {key_11}"));
    assert_failed(&o, "FAILURE", "the host's own answer broke the contract");
    assert!(
        report(&o).contains("sh-witness-fatal where=boot marker=entity_scan.sector_height"),
        "{}",
        report(&o)
    );
    let o = load_run(&path, &saved, base, &format!("stored_wrong = {key_11}, lenient = true"));
    assert_failed(&o, "FAILURE", "the host's own answer broke the contract");
    assert!(!report(&o).contains("sh-witness-fatal"), "{}", report(&o));
    // a word lowered by an arrival, and a NaN changed by one
    let o = load_run(&path, &saved, base, &format!("lowering = {key_11}"));
    assert_failed(&o, "FAILURE", "LOWERED or a NaN changed");
    let o = load_run(&path, &saved, base, "nan_change = BITS (2) .. ',' .. BITS (1)");
    assert_failed(&o, "FAILURE", "LOWERED or a NaN changed");
    // a missing expectation, a missing script name
    let o = run(
        "root = 'x', out = '@OUT@', mode = 'load', name = 'beta', campaign = 'alpha.Sch'",
        &format!("family = 'load', {SESSION_P}, {}", load_p(&saved_words_table(&saved), "")),
    );
    assert_refused_before_any_effect(&o, "needs expect=");
    let o = run(
        "root = 'x', out = '@OUT@', mode = 'load', name = 'beta'",
        &format!("family = 'load', {SESSION_P}"),
    );
    assert!(o.result.as_ref().expect_err("no campaign").contains("saved script's file name"));
    remove(&[&path]);
}

#[test]
fn each_load_fault_mode_stops_the_campaigns_own_boot_at_its_marker_after_the_first_restore_was_altered() {
    let (path, saved) = healthy_save();
    for (mode, marker) in [
        ("load_withheld", "entity_scan.callback"),
        ("load_wrong_sector", "entity_scan.sector_height"),
        ("load_wrong_state", "entity_scan.state"),
        ("load_stale_word", "entity_scan.sector_height"),
    ] {
        let o = load_run(&path, &saved, &format!("mode = '{mode}'"), "");
        o.result.as_ref().unwrap_or_else(|e| panic!("{mode}: {e}"));
        let r = report(&o);
        assert!(line_of(&r, "sh-witness-fatal").contains(&format!("where=boot marker={marker}")), "{mode}: {r}");
        assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{mode}: {r}");
        // a campaign that accepts the fault and goes on is a failure (a withheld answer is a host that raised: nothing can accept it); no restore reached is UNREACHED
        if mode != "load_withheld" {
            let o = load_run(&path, &saved, &format!("mode = '{mode}'"), "lenient = true");
            assert_failed(&o, "FAILURE", "accepted the faulty answer");
        }
        let o = load_run(&path, &saved, &format!("mode = '{mode}'"), "no_restore = true");
        assert_failed(&o, "UNREACHED", "no restore reached");
        // the fault modes consume the same whole successful save report
        let file = write_temp(&damaged(&saved, "failed"));
        let o = load_run(&file, &saved, &format!("mode = '{mode}'"), "");
        assert!(
            o.result.is_err() && o.report.is_none() && o.log.is_empty(),
            "{mode}: a failed save report is refused before the boot: {:?}",
            o.log
        );
        remove(&[&file]);
    }
    remove(&[&path]);
}
