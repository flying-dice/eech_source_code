//! Focused diagnostic-adapter check over the ACTUAL model_provider/MODEL_BRIDGE/
//! call_model converter. Boot/SCENE are scripted; no Engine boot or native claim.
use super::*;

// Existing tiny-format model installation; no copied provider/store/validator.
#[allow(clippy::expect_used, clippy::indexing_slicing)]
#[path = "../../eech-world/tests/support/model_root.rs"]
mod model_root;

const DRIVER: &str = include_str!("../../eech-world/lua/physical-flags-read-witness.lua");
const PROTOCOL: &str = include_str!("../../eech-world/lua/physical-flags-read-transaction.lua");

// Actual mlua registration of the production userdata, without constructing an
// Engine, booting C, changing globals or copying its method list into a mock.
#[test]
fn public_normal_session_driver_consumes_only_registered_userdata_methods() -> Result<(), Box<dyn std::error::Error>> {
    let lua = Lua::new();
    lua.globals().set("SESSION_PROXY", lua.create_proxy::<LuaEngine>()?)?;
    lua.load("assert(type(SESSION_PROXY.frame)=='function' and type(SESSION_PROXY.clock)=='function')")
        .exec()?;
    for (mode, tag) in [
        (MovementMode::Move, "move"),
        (MovementMode::Stop, "stop"),
        (MovementMode::ShipMove, "ship_move"),
        (MovementMode::PersonBegin, "person_begin"),
        (MovementMode::FixedWingMove, "fixed_wing_move"),
        (MovementMode::HelicopterBegin, "helicopter_begin"),
    ] {
        let value: String = lua.from_value(lua.to_value_with(&mode, options())?)?;
        assert_eq!(value, tag);
    }
    let driver = include_str!("../../eech-world/lua/normal-fire-session.lua");
    let mut methods = std::collections::BTreeSet::new();
    for (_, tail) in driver.match_indices("engine:").map(|(i, _)| (i, &driver[i + 7..])) {
        let method: String = tail.chars().take_while(|c| c.is_ascii_lowercase() || *c == '_').collect();
        if !method.is_empty() {
            methods.insert(method);
        }
    }
    assert!(methods.contains("frame") && methods.contains("clock") && methods.contains("wpn_events"));
    for method in &methods {
        let present: bool = lua.load(format!("return type(SESSION_PROXY.{method})=='function'")).eval()?;
        // The diagnostic consumes weapon-fixture's event surface. Default
        // shipping registration deliberately lacks it; do not add an API just
        // to satisfy this consumer's feature-specific contract.
        let expected = method != "wpn_events" || cfg!(feature = "weapon-fixture");
        assert_eq!(present, expected, "actual public userdata/profile mismatch: {method}");
    }
    for method in ["close", "stop", "finish", "shutdown"] {
        let absent: bool = lua.load(format!("return SESSION_PROXY.{method}==nil")).eval()?;
        assert!(absent, "public lifetime changed: {method}");
    }
    assert_eq!(methods.len(), 3);
    Ok(())
}

#[test]
fn nr602_real_delegated_refusal_preserves_the_public_converter_domain_and_reason() -> Result<(), Box<dyn std::error::Error>> {
    // Positive FIRST, then only result/reason changed, then the real canonical
    // provider's missing-record refusal. No marker is manufactured by this stub.
    for pick in ["positive", "refused", "canonical"] {
        let root = model_root::Root::minimal();
        let protocol = root.dir.join("transaction.lua");
        let output = root.dir.join("report.txt");
        std::fs::write(&protocol, PROTOCOL)?;
        let lua = Lua::new();
        let host = lua.create_table()?;
        let args = lua.create_table()?;
        for (key, value) in [
            ("root", root.dir.to_string_lossy().into_owned()),
            ("out", output.to_string_lossy().into_owned()),
            ("protocol", protocol.to_string_lossy().into_owned()),
            ("mode", "read_probe_control".into()),
            ("request_updated", "first_eligible".into()),
            ("runtime_code", "3abf03e5133279ae6ed53b657e76d4f7724f7dd3".into()),
            ("lua_sha256", "a".repeat(64)),
            ("protocol_sha256", "b".repeat(64)),
            ("bundle_sha256", "d".repeat(64)),
            ("plan_sha256", "1c472eec613c09041491617d692c91b957568a21d18d9137c316ea1cfebe7352".into()),
            ("scenario", "lebanon_retail".into()),
            ("hours", "0.001".into()),
            ("frame_ms", "100".into()),
            ("seed", "1".into()),
        ] {
            args.set(key, value)?;
        }
        host.set("args", args)?;
        lua.globals().set("host", host.clone())?;
        lua.globals().set("PICK", pick)?;
        eech_world::model_host::register(&lua, &host)?;
        eech_world::exclusive_output::register(&lua, &host)?;
        lua.set_app_data(eech_world::movement::SharedMovers(std::rc::Rc::new(std::cell::RefCell::new(
            eech_world::movement::Movers::default(),
        ))));
        lua.load(
            r#"
            local real=host.model
            DELEGATIONS=0
            host.model=function(op,scene,depth,kind,definition,key)
                if op=='scene' then return {name='SCENE'} end
                assert(op=='entity_candidate')
                DELEGATIONS=DELEGATIONS+1
                local a={result='skip',name='SCENE'}
                if PICK=='refused' then a.result='refused';a.reason='counterfactual canonical refusal' end
                if PICK=='canonical' then a=real(op,scene,depth,kind,definition,key) end
                DELEGATED=a
                return a
            end
            host.bind_mobile=function()return 'eech-world:1' end
            local noop=function()end
            host.log=noop;host.use_reference_arithmetic=noop
            host.terrain_provider=function()return noop end;host.env_provider=host.terrain_provider
            for _,n in ipairs({'move_vehicle','observe_loss','supply_reached','line_of_sight','sense','aim'}) do host[n]=noop end
        "#,
        )
        .exec()?;
        let dc = lua.create_table()?;
        dc.set("prepare_installation", lua.create_function(|_, _: LuaValue| Ok(()))?)?;
        dc.set(
            "boot",
            lua.create_function(|lua, config: LuaTable| {
                let model: LuaFunction = config.get("model")?;
                let _: LuaTable = model.call(("scene", 1))?;
                let bind: LuaFunction = config.get("bind_mobile")?;
                let descriptor = lua.create_table()?;
                descriptor.set("member_key", 1)?;
                descriptor.set("group_key", 1)?;
                let _: String = bind.call(descriptor)?;
                lua.globals().set("WRAPPED", model)?;
                let observe: LuaFunction = lua.load("return function(...) local a=WRAPPED(...);RETURNED=a;return a end").eval()?;
                let mut provider = model_provider(lua, observe)?;
                let mut facts = vec![0; 44];
                *facts.get_mut(6).ok_or_else(|| LuaError::runtime("missing mobile fact"))? = 1.0_f32.to_bits();
                match provider(ModelQuery::EntityCandidate {
                    scene: 1,
                    facts,
                    host_id: "eech-world:1".into(),
                }) {
                    Ok(ModelAnswer::EntityCandidate {
                        decision: CandidateDecision::NeedUpdated,
                        ..
                    }) => lua.globals().set("POSITIVE_CONVERTED", true)?,
                    Ok(other) => return Err(LuaError::runtime(format!("unexpected actual converted answer: {other:?}"))),
                    Err(error) => {
                        lua.globals().set("CONVERSION_ERROR", error.clone())?;
                        return Err(LuaError::runtime(error));
                    }
                }
                lua.create_table()
            })?,
        )?;
        lua.globals().set("DIAGNOSTIC_DC", dc)?;
        lua.load("package.preload.eech_dc=function()return DIAGNOSTIC_DC end").exec()?;
        let result = lua.load(DRIVER).set_name("nr602 actual READ driver").exec();
        assert!(result.is_err(), "this focused entry deliberately ends before any READ");
        let report = std::fs::read_to_string(output)?;
        assert_eq!(lua.globals().get::<i32>("DELEGATIONS")?, 1);
        assert!(report.contains("status=FAILURE") && !report.contains("status=UNREACHED"), "{report}");
        for zero in ["attempts_group=0", "attempts_read=0", "altered=0"] {
            assert!(report.contains(zero), "{report}");
        }
        if pick == "positive" {
            assert!(lua.globals().get::<bool>("POSITIVE_CONVERTED")?);
        } else {
            let error: String = lua.globals().get("CONVERSION_ERROR")?;
            assert!(
                error.contains("[model-provider:entity_candidate.refused]") && !error.contains("entity_candidate.callback"),
                "{pick}: {error}"
            );
            lua.load("assert(rawequal(RETURNED,DELEGATED) and RETURNED.result=='refused')").exec()?;
            let delegated: LuaTable = lua.globals().get("DELEGATED")?;
            let reason: String = delegated.get("reason")?;
            assert!(error.ends_with(&reason), "{pick}: {reason}: {error}");
            if pick == "canonical" {
                assert!(reason.contains("the host never issued the identity"), "{reason}");
            } else {
                assert_eq!(reason, "counterfactual canonical refusal");
            }
            assert!(!report.contains("pf-read-selection") && !report.contains("pf-read-intervention"), "{report}");
        }
    }
    Ok(())
}
