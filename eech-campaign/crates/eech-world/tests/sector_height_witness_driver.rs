//! Boundary checks of `lua/sector-height-witness.lua` (EEDCS-49, D3, Task49): the host-owned driver of the two REQUIRED witnesses ENV-SECTOR-HEIGHT-GUARDED-CLEARANCE-WITNESS-1 (M5) and
//! ENV-SECTOR-HEIGHT-NATIVE-SAVE-RESTORE-WITNESS-1 (M6). They run the REAL driver, unmodified, under a real Lua 5.1 state and the REAL exclusive-output host function, with ONE scripted collaborator standing in for the
//! campaign DLL (`require ("eech_dc")`: a boot, a frame loop and the session method `sector_height_witness_save`) and a scripted host provider. The collaborator makes the calls the real campaign makes, in the order
//! its source makes them (a task leader's step reads the sector word twice with nothing between, then asks the look-ahead and hands the minimum height to the host's move; the session save packs every sector; the load restores
//! every saved word before the first insertion) and checks every reply the way the C bridge does (mb_hostentity.h check_sector_height_reply), so an altered reply stops it with the marker the real engine stops with.
//!
//! WHAT THIS IS NOT: campaign authority. Nothing here runs the guarded `fw_move.c` branch, `store_session`, `pack_session` or `unpack_session`; the first real observation of each is the native run (docs/
//! d3-sector-height-production-contract.md section 12). These checks show that the DRIVER credits only the eligible intervention it made, never a fatal before it, never an unreached case, and that its controls and
//! every single-fault variant of them end as stated. The scripted collaborator's hand-off arithmetic is computed in Rust single precision (`HANDOFF`) and is a stand-in, not a copy of the campaign's code.
//!
//! The helper block of single-precision words in the driver is the SAME text as entity-authority.lua's (the driver runs under a hardware divide-by-zero trap natively): compared below.

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
local function facts (phase, x, z, stated)
	local w = {}
	for i = 1, 16 do w [i] = ZERO end
	w [1] = stated or ZERO
	w [12] = PHASE [phase]
	w [13], w [14] = x, z
	return table.concat (w, ",")
end

-- the host's provider (scripted): reads and restores of sector words, scans, inserts with the strict-greater raise
function PROVIDER (op, scene, depth, kind, definition, key)
	note ("model " .. tostring (op))
	if op ~= "entity_scan" then return { result = "ok" } end
	local w = split (string.match (key, "^(%S+)"), ",")
	local phase = PHASE_NAME [w [12]]
	local k = w [13] .. "," .. w [14]
	if phase == "read" then
		return { result = "height", p = w [13] .. "," .. w [14] .. "," .. (HELD [k] or ZERO) }
	elseif phase == "restore" then
		HELD [k] = w [1]
		local stored = w [1]
		if P.stored_wrong == k then stored = string.format ("0x%08x", tonumber (string.sub (w [1], 3), 16) + 1) end
		return { result = "restored", p = w [13] .. "," .. w [14] .. "," .. stored }
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

-- the reference host's movement (scripted): the physical answer depends on the minimum height handed over (half of it, exact)
function REFERENCE_MOVE (step)
	note ("move " .. tostring (step.mode))
	local y = 0
	if step.fixed_wing_terrain then y = step.fixed_wing_terrain.min_height * 0.5 end
	return { position = { 10, y, 20 } }
end

-- one task leader's step: the wrapper read, the navigation read, then (with a hand-off) the look-ahead and the move carrying fixed_wing_terrain
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
		config.move_vehicle { mode = "fixed_wing_lookahead", host_id = "ac1" }
		local min_height = HANDOFF (consumed, P.elevation, P.wp_alt)
		if P.ignores_word then min_height = HANDOFF (ZERO, P.elevation, P.wp_alt) end
		config.move_vehicle {
			mode = "fixed_wing_move", host_id = "ac1", phase = 1, elevation = P.elevation, pose = { position = { 10, 3000, 20 } },
			fixed_wing_terrain = { min_height = min_height, guide_position = { 0, 0, 0 }, sample_point = { 0, 0 }, sample_elevation = 0, site_y = 0, cg = 0 },
		}
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

local function pack (config)
	for _, sector in ipairs (P.map) do
		ask (config, "read", sector [1], sector [2])
		if P.pack_twice and sector == P.map [1] then ask (config, "read", sector [1], sector [2]) end
	end
end

local function load_boot (config)
	CONFIG_LOAD = config
	local function restore (sector)
		local word = P.saved_words [sector [1] .. "," .. sector [2]]
		if P.restore_override and P.restore_override [1] == sector [1] .. "," .. sector [2] then word = P.restore_override [2] end
		ask (config, "restore", sector [1], sector [2], word)
	end
	local function insert (item) config.model ("entity_scan", 0, nil, nil, nil, facts ("insert", item [1], item [2], item [3])) end
	config.model ("entity_scan", 0, nil, nil, nil, facts ("reset", ZERO, ZERO))
	if P.insert_first then insert (P.inserts [1]) end
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

/// run the real driver with the host args `args` (a Lua table body; `@OUT@` is the report path) and the case table `p` (a Lua table body)
fn run(args: &str, p: &str) -> Outcome {
    let out = temp_path("report");
    let _ = std::fs::remove_file(&out);
    run_to(args, p, &out)
}

fn run_to(args: &str, p: &str, out: &std::path::Path) -> Outcome {
    let out_path = out.to_string_lossy().replace('\\', "/");
    let lua = unsafe { Lua::unsafe_new() };
    lua.globals()
        .set(
            "F32_GT",
            lua.create_function(|_, (a, b): (String, String)| {
                let f = |t: &str| f32::from_bits(u32::from_str_radix(t.trim_start_matches("0x"), 16).unwrap());
                Ok(f(&a) > f(&b))
            })
            .unwrap(),
        )
        .unwrap();
    lua.globals()
        .set(
            "HANDOFF",
            lua.create_function(|_, (word, elevation, wp_alt): (String, f64, f64)| {
                // the original expressions in single precision: max (altitude, max (terrain, word - terrain)); a scripted stand-in
                let word = f32::from_bits(u32::from_str_radix(word.trim_start_matches("0x"), 16).unwrap());
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
            use_reference_arithmetic = function () end,
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
    // the one place a restore question (phase 18) is BUILT is the seeding of the host's own provider, and it is skipped when there are no seeds; the restores the campaign makes are only observed
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
    // the only call into the engine's fixture is the save method
    assert_eq!(code_text.matches("engine:sector_height_witness_save (").count(), 1);
    // the host's own provider is delegated to, never replaced: the driver assigns nothing to host.model
    assert!(!code_text.contains("host.model = ") && !code_text.contains("host.model="));
}

// ---------------------------------------------------------------------------------------------------------------- M5: the guarded clearance

#[test]
fn the_clearance_control_credits_the_first_window_that_is_followed_by_a_hand_off_and_nothing_else() {
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
    assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{r}");
    // the control stops at the first hand-off: the fourth frame is never run
    assert_eq!(o.log.iter().filter(|l| l.as_str() == "move fixed_wing_move").count(), 1, "{:?}", o.log);
}

#[test]
fn a_clearance_control_with_no_hand_off_is_unreached_never_a_pass_and_scan_reads_are_never_windows() {
    // windows with no hand-off only
    let o = clearance("mode = 'clearance_control'", "frames = { 'scan', 'step', 'step' }");
    assert_failed(&o, "UNREACHED", "no hand-off");
    assert!(
        line_of(&report(&o), "sh-witness-counts").contains("windows=2 windows_without_handoff=2 scan_reads=1"),
        "{}",
        report(&o)
    );
    // only the scan's own FIRST read: it is a scan read, not the clearance asking
    let o = clearance("mode = 'clearance_control'", "frames = { 'scan', 'scan' }");
    assert_failed(&o, "UNREACHED", "no hand-off");
    assert!(
        line_of(&report(&o), "sh-witness-counts").contains("windows=0 windows_without_handoff=0 scan_reads=2"),
        "{}",
        report(&o)
    );
}

#[test]
fn the_original_profile_characterisation_comes_first_and_the_hosted_control_must_equal_it_field_for_field() {
    // the characterisation FIRST: in the original-solver profile the host is never asked and the first hand-off is reported
    let original_out = temp_path("original");
    let _ = std::fs::remove_file(&original_out);
    let o = run_to(
        &format!("{CLEARANCE_ARGS}, mode = 'clearance_original'"),
        &format!("{CLEARANCE_P}, original = true, frames = {{ 'step', 'handoff' }}"),
        &original_out,
    );
    o.result.as_ref().expect("the original profile run passes");
    let r = report(&o);
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
    // the same setup hosted: the first hand-off equals the original profile's, field for field
    let o = clearance(
        &format!("mode = 'clearance_control', original = '{}'", original_out.to_string_lossy().replace('\\', "/")),
        "",
    );
    o.result.as_ref().expect("the hosted control equals the original profile's first hand-off");
    assert!(line_of(&report(&o), "sh-witness-original").contains("equal=1"), "{}", report(&o));
    // a hosted first hand-off that differs (another terrain elevation) is a failure naming the field
    let o = clearance(
        &format!("mode = 'clearance_control', original = '{}'", original_out.to_string_lossy().replace('\\', "/")),
        "elevation = 1600.5",
    );
    assert_failed(&o, "FAILURE", "differs from the original profile's: elevation");
    assert!(!report(&o).contains("sh-witness-original"), "{}", report(&o));
    // a run in which the host WAS asked is not the original-solver profile; one with no hand-off is UNREACHED
    let o = clearance("mode = 'clearance_original'", "");
    assert_failed(&o, "FAILURE", "this is not the original-solver profile");
    let o = clearance("mode = 'clearance_original'", "original = true, frames = { 'step', 'step' }");
    assert_failed(&o, "UNREACHED", "no hand-off");
    // the argument belongs to the hosted control only
    let o = clearance(
        &format!(
            "mode = 'clearance_changed_word', window = 2, original = '{}'",
            original_out.to_string_lossy().replace('\\', "/")
        ),
        "",
    );
    assert!(o
        .result
        .as_ref()
        .expect_err("original= for another mode")
        .contains("original=<report> is only for clearance_control"));
    assert!(o.report.is_none() && o.log.is_empty(), "{:?}", o.log);
    let _ = std::fs::remove_file(original_out);
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
fn a_changed_host_word_changes_the_campaigns_hand_off_and_the_hosts_physical_answer() {
    let control = clearance("mode = 'clearance_control'", "");
    control.result.as_ref().expect("the control passes");
    let (min, y) = (bits(1500.5), bits(750.25));
    let o = clearance(
        &format!("mode = 'clearance_changed_word', window = 2, control_min = '{min}', control_answer_y = '{y}'"),
        "",
    );
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
    // without the control's values the comparison is stated as not made, never inferred
    let o = clearance("mode = 'clearance_changed_word', window = 2", "");
    o.result.as_ref().expect("passes");
    assert!(line_of(&report(&o), "sh-witness-effect").contains("handoff=not_compared physical=not_compared"));
    // a hand-off that does not follow the changed word (the campaign ignores it) is NOT shown to: UNREACHED, never a pass
    let o = clearance("mode = 'clearance_changed_word', window = 2", "ignores_word = true");
    assert_failed(&o, "UNREACHED", "did not dominate");
    // a window that is never reached
    let o = clearance("mode = 'clearance_changed_word', window = 7", "");
    assert_failed(&o, "UNREACHED", "no hand-off in window 7");
}

#[test]
fn each_refusal_mode_is_credited_only_when_the_campaign_stops_at_its_marker_after_the_eligible_read() {
    for (mode, marker) in [
        ("clearance_withheld", "entity_scan.callback"),
        ("clearance_malformed", "entity_scan.answer"),
        ("clearance_wrong_state", "entity_scan.state"),
        ("clearance_wrong_sector", "entity_scan.sector_height"),
    ] {
        // positive first: the control of the same campaign is healthy (above); the single fault stops at its marker
        let o = clearance(&format!("mode = '{mode}', window = 2"), "");
        o.result.as_ref().unwrap_or_else(|e| panic!("{mode}: {e}"));
        let r = report(&o);
        assert!(line_of(&r, "sh-witness-fatal").contains(&format!("marker={marker}")), "{mode}: {r}");
        assert!(line_of(&r, "sh-witness-verdict").contains("status=PASS"), "{mode}: {r}");
        // a fatal BEFORE the eligible read, even at the same marker, is a failure
        let o = clearance(
            &format!("mode = '{mode}', window = 2"),
            &format!("break_at_window = 1, break_marker = '{marker}'"),
        );
        assert_failed(&o, "FAILURE", "before the eligible read of window 2");
        // a campaign that accepts the faulty answer and goes on is a failure (a withheld answer is a host that raised: nothing can accept it)
        if mode != "clearance_withheld" {
            let o = clearance(&format!("mode = '{mode}', window = 2"), "lenient = true");
            assert_failed(&o, "FAILURE", "accepted the faulty answer");
        }
        // a window that is never reached is UNREACHED
        let o = clearance(&format!("mode = '{mode}', window = 9"), "");
        assert_failed(&o, "UNREACHED", "window 9 was never reached");
    }
    // the right marker is required: a campaign whose bridge names another marker for the withheld answer is a failure
    let o = clearance("mode = 'clearance_withheld', window = 2", "callback_marker = 'entity_scan.refused'");
    assert_failed(&o, "FAILURE", "not at [model-provider:entity_scan.callback]");
}

#[test]
fn the_clearance_modes_check_their_arguments_before_any_side_effect() {
    let o = clearance("mode = 'clearance_changed_word'", "");
    assert!(o.result.as_ref().expect_err("a missing window").contains("needs window=<N>"));
    assert!(o.report.is_none() && o.log.is_empty(), "{:?}", o.log);
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
    let o = clearance("mode = 'clearance_control', scenario = 'nowhere'", "");
    assert!(o.result.as_ref().expect_err("an unknown scenario").contains("scenario="));
    assert!(o.report.is_none() && o.log.is_empty(), "{:?}", o.log);
    // an existing report is never overwritten and nothing boots
    let existing = temp_path("existing");
    std::fs::write(&existing, "earlier evidence\n").unwrap();
    let o = run_to(&format!("{CLEARANCE_ARGS}, mode = 'clearance_control'"), CLEARANCE_P, &existing);
    assert!(o.result.as_ref().expect_err("an existing report").contains("cannot create"));
    assert_eq!(std::fs::read_to_string(&existing).unwrap(), "earlier evidence\n");
    assert!(o.log.is_empty(), "{:?}", o.log);
    let _ = std::fs::remove_file(existing);
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
    assert_eq!(r.lines().filter(|l| l.starts_with("sh-save-word ")).count(), 12);
    assert!(
        r.contains("sh-save-word i=5 x=1 z=2 word=0x00000000") || r.contains("sh-save-word i=4 x=1 z=1 word=0xc0a00000"),
        "{r}"
    );
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
    let _ = std::fs::remove_file(path);
}

#[test]
fn every_single_fault_of_the_save_is_a_failure_or_a_refusal_never_a_pass() {
    let base = "mode = 'save', name = 'alpha'";
    // a seeded sector the session never packs
    let o = save_run(&format!("{base}, seeds = '9:9:0x3f800000'"), "", "save");
    assert_failed(&o, "FAILURE", "was never packed");
    // the pack asks for one sector twice
    let o = save_run(&format!("{base}, seeds = '{SEEDS}'"), "pack_twice = true", "save");
    assert_failed(&o, "FAILURE", "read a sector twice");
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
    // a host answer that breaks the contract is a failure of the host's own provider
    let o = save_run(
        &format!("{base}, seeds = '{SEEDS}'"),
        "lenient = true, stored_wrong = BITS (1) .. ',' .. BITS (1)",
        "save",
    );
    assert_failed(&o, "FAILURE", "the host's provider did not hold the seed");
    // refused before any side effect: a signalling NaN is unmeasured, a duplicate or malformed seed, a bad name
    for (args, needle) in [
        (format!("{base}, seeds = '1:1:0x7f800001'"), "signalling NaN"),
        (format!("{base}, seeds = '1:1:0xffc00001,1:1:0x3f800000'"), "seeded twice"),
        (format!("{base}, seeds = '1:1:3f800000'"), "sx:sz:0xWORD"),
        ("mode = 'save', name = '../x'".to_string(), "name=<1 to 24"),
        ("mode = 'save', name = 'aaaaaaaaaaaaaaaaaaaaaaaaa'".to_string(), "name=<1 to 24"),
        ("mode = 'save'".to_string(), "name=<1 to 24"),
    ] {
        let o = save_run(&args, "", "save");
        assert!(o.result.as_ref().expect_err(&args).contains(needle), "{args}: {:?}", o.result);
        assert!(
            o.report.is_none() && o.log.is_empty(),
            "{args}: no side effect before the arguments are accepted: {:?}",
            o.log
        );
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
            save_report_path.to_string_lossy().replace('\\', "/")
        ),
        &format!("family = 'load', {SESSION_P}, {}", load_p(&saved_words_table(report_text), extra_p)),
    )
}

#[test]
fn the_campaigns_own_load_restores_every_saved_word_exactly_before_any_insertion_and_none_is_lowered() {
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
    let _ = std::fs::remove_file(path);
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
    // an insertion before the first restore, and one between two restores: restore-before-insert is not shown
    let o = load_run(&path, &saved, base, "insert_first = true");
    assert_failed(&o, "FAILURE", "before the first restore");
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
    assert!(o.result.as_ref().expect_err("no expect").contains("needs expect="));
    assert!(o.report.is_none() && o.log.is_empty(), "{:?}", o.log);
    let o = run(
        "root = 'x', out = '@OUT@', mode = 'load', name = 'beta'",
        &format!("family = 'load', {SESSION_P}"),
    );
    assert!(o.result.as_ref().expect_err("no campaign").contains("saved script's file name"));
    let _ = std::fs::remove_file(path);
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
    }
    let _ = std::fs::remove_file(path);
}
