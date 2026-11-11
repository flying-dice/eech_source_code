//! Private real-collaborator control. No boot, public-native or natural-moving claim.
#![cfg(feature = "normal-fire-check")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(unix)]
#[link(name = "lua5.1")]
extern "C" {}
use eech_engine::{normal_fire_check::with_boundary, AimEvent, AimReply, AimRequest, AimStepKind};
use eech_engine_sys as sys;
use eech_world::{aim, cmb, flight};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
extern "C" {
    fn eech_nf_normal_check(
        directory: *const c_char,
        callback: sys::eech_aim_callback,
        environment: sys::eech_environment_callback,
        user: *mut c_void,
        output: *mut c_char,
        capacity: c_int,
    ) -> c_int;
    fn eech_nf_observer_check(output: *mut c_char, capacity: c_int) -> c_int;
}
fn event(e: flight::StepEvent) -> AimEvent {
    AimEvent {
        kind: e.kind,
        a: e.a,
        b: e.b,
        c: e.c,
        d: e.d,
        f: e.f,
    }
}
fn model_event(e: aim::Event) -> AimEvent {
    AimEvent {
        kind: e.kind,
        a: e.a,
        b: e.b,
        c: e.c,
        d: e.d,
        f: e.f,
    }
}
fn target(t: eech_engine::AimTarget) -> aim::Target {
    aim::Target {
        position: t.position,
        point: t.point,
        motion_vector: t.motion_vector,
        mobile: t.mobile,
    }
}
fn weapon(t: eech_engine::AimWeapon) -> aim::Weapon {
    aim::Weapon {
        weapon: t.weapon,
        present: t.present,
        aiming_type: t.aiming_type,
        cruise_velocity: t.cruise_velocity,
        acquire_forward: t.acquire_forward,
        source_velocity: t.source_velocity,
    }
}
fn answer(model: &mut aim::Aim, q: AimRequest) -> Result<AimReply, String> {
    let mut r = AimReply::default();
    match q {
        AimRequest::Figures(q) => {
            let weapon_id = q.weapon;
            let q = q.figures;
            let mut figures = flight::Figures {
                start_weight: q.start_weight,
                final_weight: q.final_weight,
                max_range: q.max_range,
                muzzle_velocity: q.muzzle_velocity,
                cruise_velocity: q.cruise_velocity,
                g_max: q.g_max,
                boost_time: q.boost_time,
                cruise_time: q.cruise_time,
                inhibit_time: q.inhibit_time,
                drag_coefficient: q.drag_coefficient,
                flightpath_deviation: q.flightpath_deviation,
                boost_power: q.boost_power,
                sustain_power: q.sustain_power,
                sustain_time: q.sustain_time,
                diameter: q.diameter,
                max_altitude: q.max_altitude,
                ignore_gravity: q.ignore_gravity,
                flight_profile_or_self_destr: q.flight_profile_or_self_destr,
                parachute_type: q.parachute_type,
                weapon_class: q.weapon_class,
                cruise_time_max_error: q.cruise_time_max_error,
                detonation_radius: q.detonation_radius,
                max_seeker_limit: q.max_seeker_limit,
                max_launch_angle_error: q.max_launch_angle_error,
                survive_ground_impact: q.survive_ground_impact,
                in_flight_collision_check: q.in_flight_collision_check,
                guidance_type: q.guidance_type,
                aiming_type: q.aiming_type,
                decoy_type: q.decoy_type,
                warhead_type: q.warhead_type,
                hellfire_ii: q.hellfire_ii,
                smoke_grenade: q.smoke_grenade,
                submunition_count: q.submunition_count,
                submunition_velocity_factor: q.submunition_velocity_factor,
                muzzle_velocity_max_error: q.muzzle_velocity_max_error,
                circular_error_probable: q.circular_error_probable,
                rate_of_fire: q.rate_of_fire,
                inverse_rate_of_fire: q.inverse_rate_of_fire,
                acquire_forward: q.acquire_forward,
                name: [0; flight::NAME_MAX + 1],
            };
            figures.set_name(&q.name)?;
            model.set_figures(weapon_id, &figures)?;
        }
        AimRequest::Forecast(q) => {
            let o = cmb::forecast(&cmb::ForecastInput {
                weapon_warhead: q.weapon_warhead,
                warhead: q.warhead,
                damage_capability: q.damage_capability,
                armor_penetration_capability: q.armor_penetration_capability,
                burst_duration: q.burst_duration,
                armour_level: q.armour_level,
                not_ai: q.target_not_ai,
                player_modifier: q.player_modifier,
                damage_modifier: q.damage_modifier,
                velocity: q.velocity,
            })?;
            r.forecast_capability = o.capability;
            if q.echo {
                let mut f = [0.; 12];
                f[..4].copy_from_slice(&[o.damage_modifier, o.penetration, o.armour, o.random_modifier]);
                r.echo = Some(vec![AimEvent {
                    kind: 9,
                    a: o.path,
                    b: o.raw,
                    c: o.warhead,
                    d: o.capability,
                    f,
                }]);
            }
        }
        AimRequest::Release(q) => {
            let o = model.release(
                q.weapon,
                &flight::ReleaseInput {
                    delta_time: q.delta_time,
                    current_time: q.current_time,
                    last_shot: q.last_shot,
                    not_ai: q.not_ai,
                },
            )?;
            r.release_refuse = o.refuse;
            r.release_burst_size = o.burst_size;
            r.release_last_shot = o.last_shot;
        }
        AimRequest::ReadyUpdate(q) => {
            let (s, p) = model.ready_update(q.state, q.rate, q.delta_time)?;
            r.ready_state = s;
            r.phase = p;
        }
        AimRequest::ReadyCommand(q) => {
            r.ready_state = model.ready_command(q.state, q.open)?;
        }
        AimRequest::Required(q) => {
            let (h, p) = model.required(
                q.device_attitude,
                q.intercept_vector,
                q.limits.map(|l| aim::Limits {
                    heading: [l.heading_min, l.heading_max],
                    pitch: [l.pitch_min, l.pitch_max],
                    climb: l.climb,
                }),
                q.tracking.map(|t| aim::Track {
                    device_position: t.device_position,
                    point: t.point,
                    pitch: t.pitch,
                }),
            )?;
            r.required_heading = h;
            r.required_pitch = p;
        }
        AimRequest::Step(q) => {
            let o = model.step(
                [q.current_heading, q.current_pitch],
                [q.required_heading, q.required_pitch],
                [q.heading_min, q.heading_max],
                [q.heading_rate, q.pitch_rate],
                q.delta_time,
            )?;
            r.delta_heading = o[0];
            r.delta_pitch = o[1];
            r.new_heading = o[2];
            r.new_pitch = o[3];
        }
        AimRequest::Vectors(q) => {
            let w = q.weapon.map(weapon);
            let t = q.target.map(target);
            let world = aim::World {
                map_min: q.world.map_min,
                map_max: q.world.map_max,
                comms_weapon_lag: q.world.comms_weapon_lag,
            };
            match model.vectors(
                q.device_position,
                q.device_attitude,
                q.tracking,
                w.as_ref(),
                t.as_ref(),
                &world,
                q.source_gunship,
                q.draw,
                q.echo,
            )? {
                aim::Outcome::NeedDraw => r.need_draw = true,
                aim::Outcome::Done(o) => {
                    r.result = o.valid;
                    r.weapon_vector = o.weapon_vector;
                    r.target_vector = o.target_vector;
                    r.intercept_vector = o.intercept_vector;
                    r.intercept_range = o.intercept_range;
                    r.echo = o.echo.map(|e| e.into_iter().map(model_event).collect());
                }
            }
        }
        AimRequest::Intercept(q) => {
            let o = model.intercept(q.device_position, &weapon(q.weapon), &target(q.target), q.echo)?;
            r.intercept_point = o.intercept_point;
            r.echo = o.echo.map(|e| e.into_iter().map(model_event).collect());
        }
        AimRequest::WeaponStep(q) => {
            let birth = q.kind == AimStepKind::Birth;
            let s = q.state;
            let input = flight::StepInput {
                delta_time: q.delta_time,
                system_time: q.system_time,
                position: s.position,
                attitude: s.attitude,
                motion_vector: s.motion_vector,
                velocity: s.velocity,
                weapon_lifetime: s.weapon_lifetime,
                decoy_timer: s.decoy_timer,
                missile_phase: s.missile_phase,
                loal_mode: s.loal_mode,
                weapon_on_ground: s.weapon_on_ground,
                parachute_status: s.parachute_status,
                kill_code: s.kill_code,
                entity_samples: s.entity_samples,
                has_launcher: s.has_launcher,
                has_target: s.has_target,
                server: s.server,
                submunition: s.submunition,
                coefficient: s.coefficient,
                effectiveness: s.effectiveness,
                map_min: q.map_min,
                map_max: q.map_max,
                echo: q.echo,
            };
            let answers = q
                .answers
                .into_iter()
                .map(|e| flight::StepEvent {
                    kind: e.kind,
                    a: e.a,
                    b: e.b,
                    c: e.c,
                    d: e.d,
                    f: e.f,
                })
                .collect::<Vec<_>>();
            let o = if birth {
                model.weapon_birth(q.weapon, &input, &answers, &q.draws)?
            } else if q.kind == AimStepKind::Step {
                model.weapon_step(q.weapon, &input, &answers, &q.draws)?
            } else {
                return Err(format!("private normal-fire control unsupported step {:?}", q.kind));
            };
            if o.actions.iter().any(|a| a.f.iter().any(|v| !v.is_finite())) {
                eprintln!("private step nonfinite actions={:?}; input={input:?}; answers={answers:?}", o.actions);
                return Err("private step nonfinite output; full fixture diagnostic on stderr".into());
            }
            r.need_lookup = o.state == flight::StepState::NeedLookup;
            r.need_draw = o.state == flight::StepState::NeedDraw;
            r.actions = o.actions.into_iter().map(event).collect();
            r.lookups = o.lookups.into_iter().map(event).collect();
            if q.echo {
                r.echo = Some(o.echo.into_iter().map(event).collect());
            }
        }
        other => return Err(format!("private normal-fire control does not serve {other:?}")),
    }
    Ok(r)
}
#[test]
fn real_normal_fire_uses_model_devices_and_real_d2_before_correlated_birth() {
    let dir = std::env::temp_dir().join(format!("eech-normal-fire-d2-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let path = CString::new(dir.to_str().unwrap()).unwrap();
    let mut output = vec![0_i8; 48000];
    let mut model = aim::Aim::new();
    let mut requests = std::collections::BTreeMap::new();
    let mut host = |q: AimRequest| {
        let key = match &q {
            AimRequest::Figures(_) => "figures",
            AimRequest::Forecast(_) => "forecast",
            AimRequest::ReadyUpdate(_) => "ready",
            AimRequest::ReadyCommand(_) => "command",
            AimRequest::Required(_) => "required",
            AimRequest::Step(_) => "step",
            AimRequest::Vectors(_) => "vectors",
            AimRequest::Intercept(_) => "intercept",
            AimRequest::Release(_) => "release",
            AimRequest::WeaponStep(q) if q.kind == AimStepKind::Birth => "birth",
            AimRequest::WeaponStep(_) => "flight",
            _ => "other",
        };
        *requests.entry(key).or_insert(0_usize) += 1;
        answer(&mut model, q)
    };
    let mut field = eech_world::env::Field::new().unwrap();
    let mut f = [0.; eech_world::env::FLOATS];
    f[1] = 120.;
    f[11] = 1.;
    f[12] = 4.;
    field.import(&eech_world::env::State { f, n: [1, 1, 0, 0, 0] }).unwrap();
    field.init_fixed(1).unwrap();
    let environment: eech_engine::EnvProvider = Box::new(move |q| {
        if q.op == eech_engine::EECH_ENV_WIND {
            let wind = field.wind(q.x, q.z, q.want_vector != 0)?;
            return Ok(eech_engine::EnvReply {
                speed: wind.speed,
                vector: wind.vector,
                ..Default::default()
            });
        }
        if q.op != eech_engine::EECH_ENV_SOUND_LIFETIME_INIT || !(1..=16).contains(&q.sound_count) {
            return Err(format!(
                "private normal-fire control unsupported environment op={} count={}",
                q.op, q.sound_count
            ));
        }
        let samples = q
            .sound_rate
            .iter()
            .copied()
            .zip(q.sound_size.iter().copied())
            .take(q.sound_count as usize)
            .collect::<Vec<_>>();
        let lifetime = eech_world::env::sound_lifetime_init(q.sound_looping != 0, q.sound_music != 0, &samples)?;
        // Plain numeric C reply; actual converter checks this before C applies it.
        Ok(eech_engine::EnvReply {
            sound_lifetime: lifetime,
            ..Default::default()
        })
    });
    std::env::set_current_dir(&dir).unwrap();
    let code = with_boundary(&mut host, environment, |callback, env, user| unsafe {
        eech_nf_normal_check(path.as_ptr(), callback, env, user, output.as_mut_ptr(), output.len() as c_int)
    });
    let output = unsafe { CStr::from_ptr(output.as_ptr()) }.to_str().unwrap();
    println!("{output}\nactual D2 requests={requests:?}");
    assert_eq!(code, 0, "normal-fire collaborator: {output}; requests={requests:?}");
    for key in ["forecast", "ready", "vectors", "release", "birth"] {
        assert!(requests.get(key).copied().unwrap_or(0) > 0, "missing real {key}");
    }
    // The actual C-produced coherent stream is the FIRST codec positive.
    let events = output
        .lines()
        .filter(|line| line.starts_with("normal-ev "))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    let lua = mlua::Lua::new();
    let module: mlua::Table = lua.load(include_str!("../../eech-world/lua/normal-fire-correlation.lua")).eval().unwrap();
    lua.globals().set("N", module).unwrap();
    lua.globals().set("EVENTS", events.clone()).unwrap();
    lua.load(
        r#"
      local c=N.new('private-real-collaborator',100);c:feed(1,0,EVENTS);local r=c:finish()
      assert(r.status=='CORRELATED' and r.births==1 and #r.fires==1)
      local f=r.fires[1];assert(f.launcher=='eech-world:1' and f.target=='eech-world:2' and f.frame==1)
      local b=f.birth_list[1];assert(b.w==1 and b.detachment.ammo_before=='2' and b.detachment.ammo_after=='1')
      assert(#r.steps==2 and #r.steps[1].queries==0 and #r.steps[2].queries==1 and #r.queries==1)
      assert(r.steps[1].birth==b and r.steps[2].birth==b and r.queries[1].birth==b)
      assert(r.queries[1].w==b.w and #r.queries[1].models==0) -- Original model profile
      -- The real C formatter's explicit `none` domain: no invented canonical
      -- identity, yet a well-formed normal path remains observable/ineligible.
      local unrelated=EVENTS:gsub('eech%-world:[12]','none')
      local u=N.new('private-noncanonical-control',100);u:feed(1,0,unrelated);local ur=u:finish()
      assert(ur.births==1 and #ur.queries==1 and ur.fires[1].canonical==false)
    "#,
    )
    .exec()
    .unwrap();
    // Single-field faults of that actual stream, never an alternate actor/model.
    let birth = events.lines().find(|line| line.contains("kind=normal_birth")).unwrap();
    let ineligible = events.lines().find(|line| line.contains("kind=normal_detach fire=0")).unwrap();
    let mutants = [
        (
            "ineligible malformed identity",
            events.replace(ineligible, &ineligible.replace("launcher=none", "launcher=garbage")),
        ),
        (
            "identity",
            events.replace(birth, &birth.replace("launcher=eech-world:1", "launcher=eech-world:999")),
        ),
        ("context", events.replace(birth, &birth.replacen("tick=0", "tick=1", 1))),
        ("unrelated weapon", events.replace(birth, &birth.replace("weapon=70", "weapon=71"))),
        ("malformed", events.replace(birth, &format!("{birth} extra=1"))),
    ];
    for (name, bad) in mutants {
        assert_ne!(bad, events, "mutant {name} did not change the actual stream");
        lua.globals().set("BAD", bad).unwrap();
        lua.load(
            r#"local c=N.new('private-real-collaborator',100);local ok,e=pcall(function()c:feed(1,0,BAD)end);assert(not ok)
          local ok2,e2=pcall(function()c:finish()end);assert(not ok2 and e2==e)
          local ok3,e3=pcall(function()c:feed(1,0,'')end);assert(not ok3 and e3==e)"#,
        )
        .exec()
        .unwrap();
    }
    lua.load(
        r#"
      local c=N.new('empty-private-control',100);c:feed(1,0,'');assert(c:finish().status=='UNREACHED')
      local c=N.new('failed-private-control',100);c:feed(1,0,'');local ok,e=pcall(function()c:failure('unrelated native error')end);assert(not ok)
      local ok2,e2=pcall(function()c:finish()end);assert(not ok2 and e2==e)
      local c=N.new('changed-frame-control',100);c:feed(1,0,'');assert(not pcall(function()c:feed(2,0,'')end))
    "#,
    )
    .exec()
    .unwrap();
    let mut observation = vec![0_i8; 32000];
    let observer = unsafe { eech_nf_observer_check(observation.as_mut_ptr(), observation.len() as c_int) };
    let observation = unsafe { CStr::from_ptr(observation.as_ptr()) }.to_str().unwrap();
    println!("{observation}");
    assert_eq!(observer, 0, "{observation}");
    println!("portable_lua_controls=10 actual_positive_first single_field_faults=5 noncanonical_ineligible=1 no_eligible failure_precedence frame_relation; observer_modes=4");
}
