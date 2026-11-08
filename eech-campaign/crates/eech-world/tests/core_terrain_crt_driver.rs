//! Observer orchestration only: unchanged-client delegation, no retail/native claim.
use mlua::{Lua, Table};
use std::sync::atomic::{AtomicUsize, Ordering};
const DRIVER: &str = include_str!("../lua/core-terrain-crt-observer.lua");
const REPORT: &str = "core-crt-summary schema=1 complete=1 problem=0 loads=1 ends=1 seeds=2 draws=3 pre=0 load=2 gap=1 digest=0x12345678 sites=4 marks=9 skins=1\ncore-crt-mark kind=seed order=1 draws=0 a=1 b=10 c=0\ncore-crt-mark kind=load_begin order=2 draws=0 a=1 b=2 c=0\ncore-crt-mark kind=ctor_begin order=3 draws=0 a=2 b=0 c=0\ncore-crt-mark kind=ctor_end order=5 draws=1 a=2 b=2 c=0\ncore-crt-mark kind=load_end order=7 draws=2 a=5 b=4 c=4\ncore-crt-mark kind=before_reinitialise order=8 draws=2 a=0 b=0 c=0\ncore-crt-mark kind=after_reinitialise order=10 draws=3 a=0 b=0 c=0\ncore-crt-mark kind=before_create order=11 draws=3 a=0 b=0 c=0\ncore-crt-mark kind=seed order=12 draws=3 a=1 b=10 c=2\ncore-crt-site file=control.c line=10 pre=0 load=0 gap=0 first_pre=0 first_load=0 first_gap=0 last=0 first_value=0 last_value=0 gap_scene=0\ncore-crt-site file=control.c line=20 pre=0 load=1 gap=0 first_pre=0 first_load=4 first_gap=0 last=4 first_value=1 last_value=1 gap_scene=0\ncore-crt-site file=control.c line=30 pre=0 load=1 gap=0 first_pre=0 first_load=6 first_gap=0 last=6 first_value=2 last_value=2 gap_scene=0\ncore-crt-site file=control.c line=40 pre=0 load=0 gap=1 first_pre=0 first_load=0 first_gap=9 last=9 first_value=3 last_value=3 gap_scene=-1\ncore-crt-skin scene=2 animation=3 season=2 normal=4 desert=0 winter=0 ctor=2 frames=4 calls=1 draws=1\n";
static SERIAL: AtomicUsize = AtomicUsize::new(0);
fn run(report: &str, knobs: &str) -> (Result<(), String>, String, i32, i32) {
    let base = std::env::temp_dir().join(format!("core-crt-driver-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir(&base).unwrap();
    let campaign = base.join("campaign.lua");
    let output = base.join("crt.txt");
    std::fs::write(&campaign, "local dc=require('eech_dc'); if FAIL then error('campaign failed') end; FIRST=dc.boot(ARGUMENT); if TWICE then dc.boot(ARGUMENT) end; assert(FIRST==SESSION)").unwrap();
    let lua = Lua::new();
    lua.load("host={args={},log=function()end}; BOOTS=0; QUERIES=0; ARGUMENT={token=42}; SESSION={core_terrain_crt_summary=function(self) assert(self==SESSION);QUERIES=QUERIES+1;return REPORT end}; DC={boot=function(arg) assert(arg==ARGUMENT);BOOTS=BOOTS+1;return SESSION end}; ORIGINAL=DC.boot;package.preload.eech_dc=function()return DC end").exec().unwrap();
    lua.globals().set("REPORT", report).unwrap();
    lua.load(knobs).exec().unwrap();
    let host: Table = lua.globals().get("host").unwrap();
    let args: Table = host.get("args").unwrap();
    for (key, value) in [
        ("scenario", "lebanon_retail".to_owned()),
        ("hours", "0.001".into()),
        ("frame_ms", "100".into()),
        ("seed", "1".into()),
        ("campaign_lua", campaign.to_string_lossy().into_owned()),
        ("crt_out", output.to_string_lossy().into_owned()),
    ] {
        args.set(key, value).unwrap();
    }
    eech_world::exclusive_output::register(&lua, &host).unwrap();
    let result = lua.load(DRIVER).exec().map_err(|e| e.to_string());
    lua.load("assert(DC.boot==ORIGINAL)").exec().unwrap();
    (
        result,
        std::fs::read_to_string(output).unwrap_or_default(),
        lua.globals().get("BOOTS").unwrap(),
        lua.globals().get("QUERIES").unwrap(),
    )
}
#[test]
fn delegates_the_same_client_boot_once_and_copies_the_complete_raw_report() {
    let (r, out, boots, queries) = run(REPORT, "");
    assert!(r.is_ok(), "{r:?}");
    assert_eq!(out, REPORT);
    assert_eq!((boots, queries), (1, 1));
    // An actual next seed can precede later boot boundaries. That valid finite
    // prefix is accepted without forcing a different seed/window or reporting0.
    let earlier = REPORT
        .lines()
        .filter(|line| !line.contains("kind=before_reinitialise") && !line.contains("kind=after_reinitialise") && !line.contains("kind=before_create"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let earlier = earlier
        .replace("marks=9", "marks=6")
        .replace("order=12", "order=9")
        .replace("first_gap=9 last=9", "first_gap=8 last=8");
    let (r, out, boots, queries) = run(&earlier, "");
    assert!(r.is_ok(), "{r:?}");
    assert_eq!(out, earlier);
    assert_eq!((boots, queries), (1, 1));
    // A coherent inactive constructor still has its real call/metadata, but no
    // skin draw. Remove that draw and shift the actual report ordinals with it.
    let inactive = REPORT
        .lines()
        .filter(|line| !line.contains("file=control.c line=20"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let inactive = inactive
        .replace("draws=3 pre=0 load=2", "draws=2 pre=0 load=1")
        .replace("sites=4", "sites=3")
        .replace("order=5 draws=1", "order=4 draws=0")
        .replace("order=7 draws=2", "order=6 draws=1")
        .replace("order=8 draws=2", "order=7 draws=1")
        .replace("order=10 draws=3", "order=9 draws=2")
        .replace("order=11 draws=3", "order=10 draws=2")
        .replace("order=12 draws=3", "order=11 draws=2")
        .replace("first_load=6 first_gap=0 last=6", "first_load=5 first_gap=0 last=5")
        .replace("first_gap=9 last=9", "first_gap=8 last=8")
        .replace("animation=3", "animation=-1")
        .replace("frames=4 calls=1 draws=1", "frames=-1 calls=1 draws=0");
    let (r, out, boots, queries) = run(&inactive, "");
    assert!(r.is_ok(), "inactive coherent report: {r:?}");
    assert_eq!(out, inactive);
    assert_eq!((boots, queries), (1, 1));
}
#[test]
fn partial_problem_count_and_order_corruptions_never_become_zero_draw_credit() {
    for (a, b) in [
        ("complete=1", "complete=0"),
        ("seeds=2", "seeds=99999999999999999999"),
        ("problem=0", "problem=1"),
        ("draws=3 pre=0", "draws=4 pre=0"),
        ("sites=4", "sites=5"),
        ("order=12 draws=3", "order=1 draws=3"),
        ("marks=9", "marks=5"),
    ] {
        let (r, out, _, _) = run(&REPORT.replacen(a, b, 1), "");
        assert!(r.is_err(), "{a}");
        assert!(out.is_empty());
    }
}
#[test]
fn failed_client_never_queries_a_poisoned_session_and_double_boot_is_refused() {
    let (r, _, boots, queries) = run(REPORT, "FAIL=true");
    assert!(r.is_err());
    assert_eq!((boots, queries), (0, 0));
    let (r, _, boots, queries) = run(REPORT, "TWICE=true");
    assert!(r.is_err());
    assert_eq!((boots, queries), (1, 0));
}

#[test]
fn complete_report_requires_each_boundary_and_full_site_skin_phase_contract() {
    let (r, out, _, _) = run(REPORT, "");
    assert!(r.is_ok(), "coherent positive must precede damage: {r:?}");
    assert_eq!(out, REPORT);
    let damages = [
        ("missing_load_begin", "kind=load_begin", "kind=unrelated"),
        ("missing_load_end", "kind=load_end", "kind=unrelated"),
        ("unknown_mark", "kind=before_create", "kind=alien"),
        ("malformed_site", "core-crt-site file=control.c line=20", "core-crt-site garbage"),
        ("malformed_skin", "core-crt-skin scene=2", "core-crt-skin garbage"),
        ("phase_count", "line=20 pre=0 load=1", "line=20 pre=0 load=2"),
        ("phase_ordinal", "first_load=4", "first_load=8"),
        ("last_ordinal", "first_load=4 first_gap=0 last=4", "first_load=4 first_gap=0 last=6"),
        ("mark_draw_join", "order=5 draws=1", "order=5 draws=2"),
        ("seed_phase", "a=1 b=10 c=2", "a=1 b=10 c=1"),
        ("ctor_balance", "kind=ctor_end order=5 draws=1 a=2", "kind=ctor_end order=5 draws=1 a=1"),
        ("skin_draws", "frames=4 calls=1 draws=1", "frames=4 calls=1 draws=0"),
        ("skin_scene", "core-crt-skin scene=2", "core-crt-skin scene=3"),
        ("skin_domain", "normal=4", "normal=-1"),
        ("canonical_integer", "line=20", "line=020"),
        ("value_domain", "first_value=1", "first_value=-1"),
        ("boundary_order", "kind=before_reinitialise", "kind=after_reinitialise"),
        ("summary_schema", "schema=1", "schema=2"),
        ("summary_seed_join", "seeds=2", "seeds=3"),
        ("family_order", "core-crt-site file=control.c line=20", "core-crt-skin file=control.c line=20"),
        ("skin_calls", "frames=4 calls=1 draws=1", "frames=4 calls=2 draws=1"),
        ("gap_skin_join", "gap_scene=-1", "gap_scene=3"),
        ("skin_season_join", "animation=3 season=2", "animation=3 season=3"),
    ];
    let mut wrongly_accepted = Vec::new();
    for (name, from, to) in damages {
        assert_eq!(REPORT.matches(from).count(), 1, "one mutation only: {name}");
        let (r, out, boots, queries) = run(&REPORT.replacen(from, to, 1), "");
        if let Err(error) = r {
            assert!(out.is_empty(), "{name}");
            let expected = match name {
                "missing_load_begin" | "missing_load_end" | "unknown_mark" | "boundary_order" => "observer boot boundary prefix/unknown kind",
                "malformed_site" => "observer row fields",
                "malformed_skin" => "observer field order/name",
                "phase_count" => "observer site last phase/count",
                "phase_ordinal" => "observer site phase ordinal",
                "last_ordinal" => "observer site last ordinal",
                "mark_draw_join" => "observer mark ordinal/draw join",
                "seed_phase" => "observer seed phase/line",
                "ctor_balance" => "observer constructor end",
                "skin_draws" => "observer skin activation/draws",
                "skin_scene" => "observer constructor skin identity",
                "skin_domain" | "value_domain" => "observer integer outside domain",
                "canonical_integer" => "noncanonical observer integer",
                "summary_schema" => "incomplete/refused observer coverage",
                "summary_seed_join" => "observer summary/mark phases",
                "family_order" => "observer row grammar",
                "skin_calls" => "observer skin activation/draws",
                "gap_skin_join" => "observer gap skin scene missing",
                "skin_season_join" => "observer constructor skin identity",
                _ => unreachable!(),
            };
            assert!(error.contains(expected), "{name}: expected {expected}, got {error}");
        } else {
            wrongly_accepted.push(name);
        }
        assert_eq!((boots, queries), (1, 1), "{name}");
    }
    println!(
        "Report completeness controls: coherent_positive=1 single_faults={} wrongly_accepted={wrongly_accepted:?}",
        damages.len()
    );
    assert!(wrongly_accepted.is_empty(), "damaged reports accepted: {wrongly_accepted:?}");
}

#[test]
fn multi_phase_final_singleton_has_its_own_last_endpoint() {
    // Real emitter event order: seed1, load2, ctor3, A4, end5, loadend6,
    // B7, A8, B9, B10, closing seed11. B9 is an interior draw, not an endpoint.
    let report = "core-crt-summary schema=1 complete=1 problem=0 loads=1 ends=1 seeds=2 draws=5 pre=0 load=1 gap=4 digest=0x12345678 sites=3 marks=6 skins=1\ncore-crt-mark kind=seed order=1 draws=0 a=1 b=10 c=0\ncore-crt-mark kind=load_begin order=2 draws=0 a=1 b=2 c=0\ncore-crt-mark kind=ctor_begin order=3 draws=0 a=2 b=0 c=0\ncore-crt-mark kind=ctor_end order=5 draws=1 a=2 b=2 c=0\ncore-crt-mark kind=load_end order=6 draws=1 a=5 b=4 c=4\ncore-crt-mark kind=seed order=11 draws=5 a=1 b=10 c=2\ncore-crt-site file=control.c line=10 pre=0 load=0 gap=0 first_pre=0 first_load=0 first_gap=0 last=0 first_value=0 last_value=0 gap_scene=0\ncore-crt-site file=control.c line=20 pre=0 load=1 gap=1 first_pre=0 first_load=4 first_gap=8 last=8 first_value=1 last_value=3 gap_scene=-1\ncore-crt-site file=control.c line=30 pre=0 load=0 gap=3 first_pre=0 first_load=0 first_gap=7 last=10 first_value=2 last_value=5 gap_scene=-1\ncore-crt-skin scene=2 animation=3 season=2 normal=4 desert=0 winter=0 ctor=2 frames=4 calls=1 draws=1\n";
    let (result, out, boots, queries) = run(report, "");
    assert!(result.is_ok(), "coherent multi-phase positive: {result:?}");
    assert_eq!(out, report);
    assert_eq!((boots, queries), (1, 1));
    let from = "first_load=4 first_gap=8 last=8";
    assert_eq!(report.matches(from).count(), 1);
    let damaged = report.replacen(from, "first_load=4 first_gap=8 last=9", 1);
    let (result, out, boots, queries) = run(&damaged, "");
    println!(
        "Endpoint controls: coherent_multiphase=1 one_field_mutation=last8_to9 accepted={}",
        result.is_ok()
    );
    let error = result.expect_err("a site's sole final-phase draw must also be its last draw");
    assert!(error.contains("observer site last ordinal"), "{error}");
    assert!(out.is_empty());
    assert_eq!((boots, queries), (1, 1));
}
