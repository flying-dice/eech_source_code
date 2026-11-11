//! TEST-ONLY (EEDCS-49, D3, Task49; the collision consumer control, both profiles): the engine's model-provider callback served by the REAL reference provider of eech-world (`host.model`, its real `SectorRoster`, flag store and model store over the
//! minimal installation of `eech-world/tests/support/model_root.rs`: scene 1 is a box of half-width 1).
//!
//! WHAT THIS IS, AND IS NOT. The shipped path between the native engine and the host's Lua provider is `eech-engine` `model_callback` and `eech-dc` `model_provider`, installed only by `Engine::boot` (which needs a retail installation and a host's Lua
//! state). This file is NOT that bridge and does not claim to be: it is a small TEST bridge, made only for the operations the collision control asks (catalogue, scene, triangle, entity scan, entity candidate, entity pose, entity collision and
//! the four flag phases), that turns the engine's request structs into the provider's own arguments (the same text places `eech-dc` `call_model` builds) and the provider's own answer tables into the reply structs. Every DECISION is the real
//! provider's (the roster, the scan kernel, the candidate kernel, the narrow phase, the flag store, the scene data); nothing is decided here. An operation outside the list is refused by name.
use eech_engine_sys as sys;
use eech_world::movement::{Kind, Movers, Pose, SharedMovers};
use eech_world::roster::{SectorRoster, SharedRoster};
use mlua::{Function, Lua, Table};
use std::cell::RefCell;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::rc::Rc;

#[path = "../../../eech-world/tests/support/model_root.rs"]
mod model_root;

struct Host {
    _lua: Lua,
    model: Function,
    roster: Rc<RefCell<SectorRoster>>,
    movers: Rc<RefCell<Movers>>,
    _root: model_root::Root,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
    /// the questions asked, by operation: scan, candidate, pose, collision, flag, other
    static ASKED: RefCell<[usize; 6]> = const { RefCell::new([0; 6]) };
    /// the flag-store reads the provider answered while the CANDIDATE decision was open (the destructive getter named by the host)
    static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// One real reference host: a Lua state with `host.model` over the minimal installation, one roster (with its flag store) and an empty mover registry.
pub fn install() {
    let root = model_root::Root::minimal();
    // The scene of the control's moving mobile is a scene WITHOUT a collision object (the original's -1, as many scenes of the original data are): the original's narrow phase then tests the body's bounding box and the host
    // answers no triangle. The minimal installation's scene 1 names its (line) object as its collision object, which the store cannot serve triangles for; the one field of the record is changed here, in the file the store reads,
    // so the model data is the store's own and no answer is forged: header 40 bytes, then the scene record's collision object at byte 128 (16 x -1, the scene index and self-shadows, four counts, three counts, the
    // approximations and the main object, the shadow approximation and shadow polygon object, the scale).
    let database = root.dir.join("cohokum").join("3ddata").join("3dobjdb.bin");
    let mut bytes = std::fs::read(&database).expect("3dobjdb.bin");
    assert_eq!(
        &bytes[168..172],
        &1i32.to_le_bytes(),
        "the minimal installation's collision object is where the record layout says"
    );
    bytes[168..172].copy_from_slice(&(-1i32).to_le_bytes());
    std::fs::write(&database, bytes).expect("3dobjdb.bin");
    let lua = Lua::new();
    let host = lua.create_table().expect("a table");
    let args = lua.create_table().expect("a table");
    args.set("root", root.dir.to_string_lossy().into_owned()).expect("root");
    host.set("args", args).expect("args");
    eech_world::model_host::register(&lua, &host).expect("the provider registers");
    let roster = Rc::new(RefCell::new(SectorRoster::default()));
    lua.set_app_data(SharedRoster(roster.clone()));
    let movers = Rc::new(RefCell::new(Movers::default()));
    lua.set_app_data(SharedMovers(movers.clone()));
    let model: Function = host.get("model").expect("host.model");
    HOST.with(|slot| {
        *slot.borrow_mut() = Some(Host {
            _lua: lua,
            model,
            roster,
            movers,
            _root: root,
        })
    });
    ASKED.with(|asked| *asked.borrow_mut() = [0; 6]);
    LOG.with(|log| log.borrow_mut().clear());
}

/// the questions asked of the provider since `install`: (scan, candidate, pose, collision, flag, other)
pub fn asked() -> [usize; 6] {
    ASKED.with(|asked| *asked.borrow())
}

pub fn log() -> Vec<String> {
    LOG.with(|log| log.borrow().clone())
}

/// One new hosted observer control: wrap the REAL Lua model provider before
/// its first entry. The wrapper counts actual calls, then delegates unchanged.
#[cfg(feature = "normal-fire-correlation")]
#[allow(dead_code)]
pub fn normal_capture_start() {
    HOST.with(|slot| {
        let mut slot = slot.borrow_mut();
        let host = slot.as_mut().expect("installed provider");
        let module: Table = host._lua.load(include_str!("../../../eech-world/lua/normal-fire-correlation.lua")).eval().unwrap();
        host._lua.globals().set("NORMAL_MODULE", module).unwrap();
        host._lua.globals().set("ORIGINAL_MODEL", host.model.clone()).unwrap();
        host.model = host._lua.load("NORMAL_CAPTURE=NORMAL_MODULE.new('private-hosted-query',100); NORMAL_MODEL_OPS={}; return function(op,...) NORMAL_MODEL_OPS[#NORMAL_MODEL_OPS+1]=op; NORMAL_CAPTURE:model_entry(op); return ORIGINAL_MODEL(op,...) end").eval().unwrap();
    });
}

#[cfg(feature = "normal-fire-correlation")]
#[allow(dead_code)]
pub fn normal_capture_check(events: &str) {
    HOST.with(|slot| {
        let slot = slot.borrow();
        let host = slot.as_ref().expect("installed provider");
        host._lua.globals().set("NORMAL_EVENTS", events).unwrap();
        host._lua
            .load(
                r#"
          NORMAL_CAPTURE:feed(1,0,NORMAL_EVENTS)
          local r=NORMAL_CAPTURE:finish()
          assert(#r.steps==1 and #r.queries==1 and #r.queries[1].models>0)
          assert(r.births==0 and r.status=='UNREACHED' and r.queries[1].birth==nil)
          assert(r.queries[1].models[1].call>1) -- real boot/roster traffic counted first
          local calls=r.model_calls
          local function reject(text)
            local c=NORMAL_MODULE.new('private-hosted-query',100)
            -- Replay the ACTUAL provider's entry operations for this codec
            -- mutation only; the coherent positive above used live delegation.
            for _,op in ipairs(NORMAL_MODEL_OPS) do c:model_entry(op) end
            local ok,e=pcall(function() c:feed(1,0,text) end);assert(not ok)
            for _,f in ipairs({function()c:finish()end,function()c:feed(1,0,'')end,function()c:model_entry('scene')end,function()c:failure('later')end}) do
              local ok2,e2=pcall(f);assert(not ok2 and e2==e)
            end
          end
          local call=NORMAL_EVENTS:match('kind=normal_model.- call=(%d+)')
          assert(call);reject((NORMAL_EVENTS:gsub('call='..call..' ', 'call=0 ',1)))
          local model=NORMAL_EVENTS:match('(normal%-ev [^\n]*kind=normal_model[^\n]*)')
          assert(model);reject((NORMAL_EVENTS:gsub(model:gsub('([^%w])','%%%1'),model:gsub(' w=1 ',' w=2 '),1)))
          reject(NORMAL_EVENTS..'normal-ev seq=999999 tick=0 kind=normal_error reason=overflow\n')
          local c=NORMAL_MODULE.new('empty-hosted-query',100);c:feed(1,0,'');assert(c:finish().status=='UNREACHED')
          print('hosted_query_positive=1 callback_identity_faults=2 observer_error=1 empty_control=1 live_model_calls='..calls)
        "#,
            )
            .exec()
            .unwrap();
    });
}

/// A fresh host binding for a scenario: the roster is emptied and the flag store replaced (scenario isolation of the test, as `flags_host::fresh`).
pub extern "C" fn fresh() {
    HOST.with(|slot| {
        let slot = slot.borrow();
        let host = slot.as_ref().expect("a host was installed");
        let mut roster = host.roster.borrow_mut();
        roster.clear();
        roster.flags = eech_world::flags::FlagStore::default();
        // the host's mover registry is fresh too: the identities of the last scenario's mobiles are gone with it
        *host.movers.borrow_mut() = Movers::default();
    });
}

/// # Safety
/// `key` is a NUL-terminated text.
pub unsafe extern "C" fn observe(key: *const c_char, array: c_int) -> u32 {
    let key = CStr::from_ptr(key).to_str().expect("a flag key is text");
    HOST.with(|slot| {
        let slot = slot.borrow();
        let host = slot.as_ref().expect("a host was installed");
        let roster = host.roster.borrow();
        roster
            .flags
            .word(key, array as usize)
            .unwrap_or_else(|refusal| panic!("the host does not know the key: {}", refusal.0))
    })
}

fn hex_text(floats: &[f32]) -> String {
    floats.iter().map(|float| format!("0x{:08x}", float.to_bits())).collect::<Vec<_>>().join(",")
}

fn words_of(text: &str) -> Vec<f32> {
    text.split(',')
        .map(|token| f32::from_bits(u32::from_str_radix(token.trim_start_matches("0x"), 16).expect("a bit pattern")))
        .collect()
}

unsafe fn put_error(error: *mut c_char, size: usize, text: &str) {
    let bytes = text.as_bytes();
    let count = bytes.len().min(size.saturating_sub(1));
    for (at, byte) in bytes[..count].iter().enumerate() {
        *error.add(at) = *byte as c_char;
    }
    *error.add(count) = 0;
}

fn put_name(reply: &mut sys::eech_model_reply, name: &str) {
    for (slot, byte) in reply.name.iter_mut().zip(name.bytes()) {
        *slot = byte as c_char;
    }
}

fn answer(
    model: &Function,
    op: &str,
    scene: Option<i32>,
    depth: Option<i32>,
    kind: Option<i32>,
    definition: Option<i32>,
    key: Option<String>,
) -> Result<Table, String> {
    model
        .call::<Table>((op, scene, depth, kind, definition, key))
        .map_err(|error| format!("[test-bridge:{op}.callback] {error}"))
}

fn text(table: &Table, field: &str) -> Option<String> {
    table.get::<Option<String>>(field).ok().flatten()
}

unsafe fn convert(request: &sys::eech_model_request, reply: &mut sys::eech_model_reply, model: &Function) -> Result<(), String> {
    let identity = CStr::from_ptr(request.host_id.as_ptr())
        .to_str()
        .map_err(|_| "[test-bridge] an identity that is not text".to_string())?;
    let with_identity = |facts: &str| {
        if identity.is_empty() {
            facts.to_string()
        } else {
            format!("{facts} {identity}")
        }
    };
    let count = usize::try_from(request.pose_count).unwrap_or(0).min(request.pose.len());
    let facts = hex_text(&request.pose[..count]);
    let bump = |at: usize| ASKED.with(|asked| asked.borrow_mut()[at] += 1);
    match request.op {
        sys::EECH_MODEL_CATALOGUE => {
            bump(5);
            let table = answer(model, "catalogue", None, None, None, None, None)?;
            reply.scene_count = table.get::<i32>("scenes").map_err(|e| e.to_string())?;
        }
        sys::EECH_MODEL_SCENE => {
            bump(5);
            let table = answer(model, "scene", Some(request.scene), None, None, None, None)?;
            put_name(reply, &text(&table, "name").unwrap_or_default());
            reply.loaded = c_int::from(table.get::<bool>("loaded").unwrap_or(false));
            reply.collider = c_int::from(table.get::<bool>("collider").unwrap_or(false));
            reply.radius = words_of(&text(&table, "radius").ok_or("[test-bridge] no radius")?)[0];
            reply.bounding_box.copy_from_slice(&words_of(&text(&table, "bb").ok_or("[test-bridge] no bb")?));
            reply
                .bounding_box_without_lines
                .copy_from_slice(&words_of(&text(&table, "bb2").ok_or("[test-bridge] no bb2")?));
        }
        sys::EECH_MODEL_TRIANGLE => {
            bump(5);
            let table = answer(model, "triangle", Some(request.scene), Some(request.depth), None, None, None)?;
            match text(&table, "result").as_deref() {
                Some("triangle") => {
                    reply.result = sys::EECH_MODEL_TRIANGLE_FOUND;
                    reply.points.copy_from_slice(&words_of(&text(&table, "p").ok_or("[test-bridge] no p")?));
                }
                Some("end") => reply.result = sys::EECH_MODEL_TRIANGLE_END,
                Some("absent") => reply.result = sys::EECH_MODEL_TRIANGLE_ABSENT,
                other => return Err(format!("[test-bridge:triangle.answer] {other:?} {:?}", text(&table, "reason"))),
            }
        }
        sys::EECH_MODEL_ENTITY_SCAN => {
            let phase = request.pose[11].to_bits();
            let flag = matches!(phase, 0x4198_0000 | 0x41a0_0000 | 0x41a8_0000 | 0x41b0_0000);
            bump(if flag { 4 } else { 0 });
            let table = answer(model, "entity_scan", Some(0), None, None, None, Some(with_identity(&facts)))?;
            let result = text(&table, "result").unwrap_or_default();
            if result == "refused" {
                return Err(format!("[test-bridge:entity_scan.refused] {}", text(&table, "reason").unwrap_or_default()));
            }
            put_name(reply, &text(&table, "name").unwrap_or_default());
            let p = text(&table, "p").map(|p| words_of(&p)).unwrap_or_default();
            reply.candidate[..p.len()].copy_from_slice(&p);
            reply.candidate_count = p.len() as c_int;
            reply.candidate_state = match result.as_str() {
                "first" => sys::EECH_MODEL_SCAN_FIRST,
                "next" => sys::EECH_MODEL_SCAN_NEXT,
                "offer" => sys::EECH_MODEL_SCAN_OFFER,
                "done_none" => sys::EECH_MODEL_SCAN_DONE_NONE,
                "done_hit" => sys::EECH_MODEL_SCAN_DONE_HIT,
                "placed" => sys::EECH_MODEL_SCAN_PLACED,
                "removed" => sys::EECH_MODEL_SCAN_REMOVED,
                "cleared" => sys::EECH_MODEL_SCAN_CLEARED,
                "height" => sys::EECH_MODEL_SCAN_HEIGHT,
                "restored" => sys::EECH_MODEL_SCAN_RESTORED,
                "flag_group" => sys::EECH_MODEL_SCAN_FLAG_GROUP,
                "flag_set" => sys::EECH_MODEL_SCAN_FLAG_SET,
                "flag_read" => sys::EECH_MODEL_SCAN_FLAG_READ,
                "flag_reset" => sys::EECH_MODEL_SCAN_FLAG_RESET,
                other => {
                    return Err(format!(
                        "[test-bridge:entity_scan.answer] the result {other:?} is not one this test bridge carries"
                    ))
                }
            };
        }
        sys::EECH_MODEL_ENTITY_CANDIDATE => {
            bump(1);
            let table = answer(model, "entity_candidate", Some(request.scene), None, None, None, Some(with_identity(&facts)))?;
            let result = text(&table, "result").unwrap_or_default();
            if result == "refused" {
                return Err(format!("[test-bridge:entity_candidate.refused] {}", text(&table, "reason").unwrap_or_default()));
            }
            put_name(reply, &text(&table, "name").unwrap_or_default());
            reply.candidate_state = match result.as_str() {
                "skip" => sys::EECH_MODEL_CANDIDATE_SKIP,
                "need_updated" => {
                    LOG.with(|log| log.borrow_mut().push("need_updated".into()));
                    sys::EECH_MODEL_CANDIDATE_NEED_UPDATED
                }
                "need_normal" => sys::EECH_MODEL_CANDIDATE_NEED_NORMAL,
                "buildings" => sys::EECH_MODEL_CANDIDATE_BUILDINGS,
                "need_fixed" => sys::EECH_MODEL_CANDIDATE_NEED_FIXED,
                "need_placement" => sys::EECH_MODEL_CANDIDATE_NEED_PLACEMENT,
                "test" => {
                    let pose = words_of(&text(&table, "p").ok_or("[test-bridge] no pose")?);
                    reply.candidate[..pose.len()].copy_from_slice(&pose);
                    reply.candidate_count = pose.len() as c_int;
                    LOG.with(|log| log.borrow_mut().push("test".into()));
                    sys::EECH_MODEL_CANDIDATE_TEST
                }
                other => {
                    return Err(format!(
                        "[test-bridge:entity_candidate.answer] the result {other:?} is not one this test bridge carries"
                    ))
                }
            };
        }
        sys::EECH_MODEL_ENTITY_POSE => {
            bump(2);
            let table = answer(
                model,
                "entity_pose",
                Some(0),
                Some(request.depth),
                Some(request.kind),
                Some(request.definition),
                Some(with_identity(&facts)),
            )?;
            let result = text(&table, "result").unwrap_or_default();
            reply.candidate_state = match result.as_str() {
                "recorded" => sys::EECH_MODEL_POSE_RECORDED,
                "unrecorded" => sys::EECH_MODEL_POSE_UNRECORDED,
                "retired" => sys::EECH_MODEL_POSE_RETIRED,
                "unknown" => sys::EECH_MODEL_POSE_UNKNOWN,
                "no_id" => sys::EECH_MODEL_POSE_NO_ID,
                other => return Err(format!("[test-bridge:entity_pose.answer] {other:?} {:?}", text(&table, "reason"))),
            };
            reply.candidate_count = table.get::<Option<c_int>>("kind").ok().flatten().unwrap_or(0);
            reply.write_mask = table.get::<Option<c_int>>("difference").ok().flatten().unwrap_or(0);
        }
        sys::EECH_MODEL_ENTITY_COLLISION => {
            bump(3);
            let table = answer(model, "entity_collision", Some(request.scene), None, None, None, Some(facts))?;
            let result = text(&table, "result").unwrap_or_default();
            match result.as_str() {
                "hit" => reply.result = sys::EECH_MODEL_TRIANGLE_FOUND,
                "miss" => reply.result = sys::EECH_MODEL_TRIANGLE_END,
                other => return Err(format!("[test-bridge:entity_collision.answer] {other:?} {:?}", text(&table, "reason"))),
            }
            put_name(reply, &text(&table, "name").unwrap_or_default());
            reply.write_mask = table.get::<Option<c_int>>("mask").ok().flatten().unwrap_or(0);
            reply.points[..6].copy_from_slice(&words_of(&text(&table, "p").ok_or("[test-bridge] no p")?));
        }
        other => {
            return Err(format!(
                "[test-bridge:callback] the collision control's test bridge carries no operation {other}"
            ))
        }
    }
    Ok(())
}

/// # Safety
/// The pointers are the engine's request, reply and error buffer (`campaign_model_fn`).
pub unsafe extern "C" fn model(
    request: *const sys::eech_model_request,
    reply: *mut sys::eech_model_reply,
    error: *mut c_char,
    size: usize,
    _user: *mut c_void,
) -> c_int {
    let request = &*request;
    let reply = &mut *reply;
    let result = HOST.with(|slot| {
        let slot = slot.borrow();
        let host = slot.as_ref().ok_or_else(|| "[test-bridge] no host was installed".to_string())?;
        unsafe { convert(request, reply, &host.model) }
    });
    match result {
        Ok(()) => 0,
        Err(reason) => {
            put_error(error, size, &reason);
            1
        }
    }
}

/// the host identities the test collaborator has issued; never reused, so a scenario's identities are new to the campaign's roster whatever the roster remembers
static NEXT_HOST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// The C `campaign_member_descriptor` (gp_roster.h), field for field: what the campaign tells the host when it links a movable member.
#[repr(C)]
pub struct MemberDescriptor {
    member_key: u64,
    group_key: u64,
    entity_index: c_int,
    entity_type: c_int,
    sub_type: c_int,
    side: c_int,
    has_pose: c_int,
    position: [f32; 3],
    velocity: f32,
    attitude: [f32; 9],
    movement_kind: c_int,
    motion_vector: [f32; 3],
    rotor: [f32; 3],
}

/// THE PRIVATE TEST COLLABORATOR of the campaign's roster (`campaign_roster_host.bind_member`): the host's own supported registration path, as the reference host's `host.bind_mobile` does it (main.rs): a member the campaign links is
/// issued the next identity `eech-world:<n>` and, when it arrives with a pose, a mover RECORD is started from that pose and its motion vector (`Movers::bind`), the very record the host's candidate decision reads. Only the descriptor's
/// kind of movement this control needs (a fixed wing) is carried; anything else is refused by name.
///
/// # Safety
/// The pointers are the roster's descriptor and its identity and error buffers.
pub unsafe extern "C" fn bind_member(
    descriptor: *const MemberDescriptor,
    host_id: *mut c_char,
    size: usize,
    error: *mut c_char,
    error_size: usize,
    _user: *mut c_void,
) -> c_int {
    let descriptor = &*descriptor;
    if descriptor.has_pose == 0 || descriptor.movement_kind != 4 {
        put_error(
            error,
            error_size,
            "[test-bridge:bind_mobile] the test collaborator binds only a fixed wing that arrives with a pose",
        );
        return 1;
    }
    let id = format!("eech-world:{}", NEXT_HOST_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
    let pose = Pose {
        position: descriptor.position,
        velocity: descriptor.velocity,
        attitude: descriptor.attitude,
    };
    let bound = HOST.with(|slot| {
        let slot = slot.borrow();
        let host = slot.as_ref().ok_or_else(|| "[test-bridge:bind_mobile] no host was installed".to_string())?;
        let result = host.movers.borrow_mut().bind(&id, Kind::FixedWing, pose, Some(descriptor.motion_vector), None);
        result
    });
    if let Err(reason) = bound {
        put_error(error, error_size, &reason);
        return 1;
    }
    let bytes = id.as_bytes();
    if bytes.len() + 1 > size {
        put_error(error, error_size, "[test-bridge:bind_mobile] the identity does not fit");
        return 1;
    }
    for (at, byte) in bytes.iter().enumerate() {
        *host_id.add(at) = *byte as c_char;
    }
    *host_id.add(bytes.len()) = 0;
    0
}
