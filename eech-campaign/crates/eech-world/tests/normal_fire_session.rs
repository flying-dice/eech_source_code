//! Portable Lua transport/reference controls, run by the real host executable.
//! The report is explicitly synthetic; Engine is a stand-in here. Actual C
//! normal-fire/D2 and hosted-provider controls are separate, not relabelled.
use std::process::Command;

#[test]
fn normal_session_validates_the_whole_reference_before_effects_and_failure_precedes_unreached() {
    let root = std::env::temp_dir().join(format!("eech-normal-session-codec-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let positive = root.join("synthetic-positive.txt");
    std::fs::write(&positive, include_bytes!("data/synthetic-normal-fire-positive.txt")).unwrap();
    let damaged = root.join("synthetic-damaged.txt");
    std::fs::write(
        &damaged,
        include_str!("data/synthetic-normal-fire-positive.txt").replace("carry=13", "carry=14"),
    )
    .unwrap();
    let path = |p: &std::path::Path| format!("{:?}", p.to_string_lossy().replace('\\', "/"));
    let lua = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("lua");
    let script = format!(
        r#"
local base={base};local positive={positive};local damaged={damaged};local root={root}
local K=assert(loadfile(base..'/normal-fire-session-contract.lua'))()
local N=assert(loadfile(base..'/normal-fire-correlation.lua'))();K.event=N.event
local T=assert(loadfile(base..'/physical-flags-read-transaction.lua'))()
local f=assert(io.open(positive,'rb'));local text=f:read('*a');f:close()
assert(K.healthy(K.parse(text),{{}},T).target=='eech-world:2') -- coherent codec positive FIRST
for _,pair in ipairs({{{{'target=eech%-world:2','target=eech-world:3'}},{{'carry=13','carry=14'}},{{'failed=0','failed=1'}},{{'frames=1500','frames=1499'}},{{'position=10,100,20','position=11,100,20'}}}}) do
 local bad,n=text:gsub(pair[1],pair[2],1);assert(n==1 and not pcall(function()K.healthy(K.parse(bad),{{}},T)end))
end
local calls={{prepare=0,boot=0,frame=0,close=0,output=0}};local fatal=false
package.preload.eech_dc=function()return {{
 prepare_installation=function()calls.prepare=calls.prepare+1 end,
 boot=function(options)
  calls.boot=calls.boot+1;local count=0
  assert(type(options.aim)=='function' and type(options.model)=='function' and type(options.move_vehicle)=='function')
  return {{frame=function(_,ms)assert(ms==100);count=count+1;calls.frame=count;if fatal and count==10 then error('unrelated native stand-in failure')end end,
   clock=function()return {{elapsed_seconds=count/10}}end,
   wpn_events=function()return ''end,
   close=function()calls.close=calls.close+1 end}}
 end}}end
local create=host.create_exclusive;host.create_exclusive=function()calls.output=calls.output+1;error('output touched')end
local function args(mode,out,reference)
 return {{root=root,out=root..out,mode=mode,capture='fixture:'..mode,runtime=string.rep('b',40),bundle=string.rep('a',64),driver_sha256=string.rep('a',64),collector=base..'/normal-fire-correlation.lua',collector_sha256=string.rep('a',64),transaction=base..'/physical-flags-read-transaction.lua',transaction_sha256=string.rep('a',64),contract=base..'/normal-fire-session-contract.lua',contract_sha256=string.rep('a',64),reference=reference,reference_sha256=reference and string.rep('c',64) or nil}}
end
host.args=args('terrain_lift','/positive-gate.txt',positive)
local ok,why=pcall(assert(loadfile(base..'/normal-fire-session.lua')))
assert(not ok and tostring(why):find('output touched',1,true) and calls.output==1 and calls.prepare==0)
calls.output=0
host.args=args('terrain_lift','/forbidden.txt',damaged)
assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
assert(calls.output==0 and calls.prepare==0 and calls.boot==0 and calls.frame==0)
host.create_exclusive=create
-- Only the Engine/terrain factories are stand-ins in these EMPTY/FAILURE
-- transport controls. No synthetic firing/provider result is credited.
host.terrain_provider=function()return function()error('unused terrain')end end
host.env_provider=function()return function()error('unused environment')end end
host.args=args('healthy','/empty.txt')
assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
assert(calls.frame==1500 and calls.close==1)
local e=assert(io.open(root..'/empty.txt','rb'));local r=K.parse(e:read('*a'));e:close()
assert(r.verdict.status=='UNREACHED' and r.counts.failed=='0' and r.counts.changed=='0')
fatal=true;calls.close=0;calls.frame=0;host.args=args('healthy','/failed.txt')
assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
assert(calls.frame==10 and calls.close==0)
local e=assert(io.open(root..'/failed.txt','rb'));local r=K.parse(e:read('*a'));e:close()
assert(r.verdict.status=='FAIL' and r.counts.failed=='1')
-- Coherent whole-driver transport positive then B. This stand-in delivers the
-- explicitly SYNTHETIC codec stream, not a real normal campaign firing claim.
local baseline=K.parse(text);local unregistered=false;local options,binds,candidate_calls,y,lift_calls
local function patterns(row)return table.concat(row,',')end
local Z='0x00000000';local O='0x3f800000'
local function flag_request(phase,lane)
 local w={{}};for i=1,16 do w[i]=Z end
 w[12]=phase==19 and '0x41980000' or '0x41a80000'
 if phase==19 then w[1]=O else w[14]=O end
 return patterns(w)..(phase==21 and ' f1' or '')
end
local function candidate_key(carried,placed)
 local w=K.patterns(baseline.selection.request,44);w[15]=y==100 and '0x42c80000' or '0x44160000'
 if carried then w[21]=O end
 if placed then w[44]=O;w[28]=O;w[32]=O;w[36]=O end
 return patterns(w)..' eech-world:2'
end
host.bind_mobile=function()binds=binds+1;return 'eech-world:'..binds end
host.move_vehicle=function(step)
 assert(step.host_id=='eech-world:2' and step.mode=='move');y=step.terrain.elevation
 if y==600 then lift_calls=lift_calls+1 else assert(y==100) end
 return {{position={{10,y,20}},motion_vector={{2,0,0}}}}
end
host.model=function(op)
 if op=='scene' then return {{name='fixture-scene1'}} end
 if op=='entity_scan' then
  if candidate_calls==0 then return {{result='flag_group',name='f1',p=O..','..Z..','..Z}} end
  return {{result='flag_read',p=Z..','..O..','..Z}}
 end
 if op=='entity_candidate' then
  candidate_calls=candidate_calls+1
  if candidate_calls<3 then return {{result=candidate_calls==1 and 'need_updated' or 'need_placement',name='fixture-scene1'}} end
  local p=K.patterns(baseline.selection.pose,16);if y==600 then p[2]='0x44160000';p[5]='0x44160000' end
  return {{result='test',name='fixture-scene1',p=patterns(p)}}
 end
 return {{result='recorded'}}
end
package.loaded.eech_dc=nil
package.preload.eech_dc=function()return {{prepare_installation=function()end,boot=function(o)
 options=o;binds=0;candidate_calls=0;y=100;lift_calls=0
 o.bind_mobile({{member_key=1,group_key=1}});o.bind_mobile({{member_key=2,group_key=1}})
 o.model('scene',1);o.model('entity_scan',0,nil,nil,nil,flag_request(19))
 for _=1,8 do o.model('scene',1) end
 local count=0;local pending=''
 return {{clock=function()return {{elapsed_seconds=count/10}}end,close=function()end,
 frame=function(_,ms)
  assert(ms==100);count=count+1
  if count==301 then
   local step={{host_id='eech-world:2',mode='move',terrain={{elevation=100}},map_max={{4095,10000,4095}}}}
   o.move_vehicle(step);assert(step.terrain.elevation==100,'caller input mutated')
   o.model('entity_candidate',1,nil,nil,nil,candidate_key(false,false))
   o.model('entity_scan',0,nil,nil,nil,flag_request(21))
   o.model('entity_candidate',1,nil,nil,nil,candidate_key(true,false))
   o.model('entity_pose',1)
   o.model('entity_candidate',1,nil,nil,nil,candidate_key(true,true))
   local e=baseline.evidence
   pending=K.unhex(e.fire)..'\n'..K.unhex(e.detach)..'\n'..K.unhex(e.birth)..'\nnormal-ev seq=4 tick=30000 kind=normal_end fire=1 births=1\n'
   local tail=K.unhex(e.step)..'\n'..K.unhex(e.query)..'\n'..K.unhex(e.models)..K.unhex(e.query_end)..'\n'..K.unhex(e.step_end)..'\n'
   pending=pending..tail:gsub('seq=(%d+)',function(n)return 'seq='..(tonumber(n)+1)end)
   if unregistered then pending=pending:gsub('eech%-world:[12]','none') end
  end
 end,wpn_events=function()local v=pending;pending='';return v end}}
 end}}end
host.args=args('healthy','/transport-positive.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/transport-positive.txt','rb'));local healthy=e:read('*a');e:close()
assert(K.healthy(K.parse(healthy),{{}},T) and lift_calls==0)
host.args=args('terrain_lift','/transport-B.txt',root..'/transport-positive.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/transport-B.txt','rb'));local b=K.parse(e:read('*a'));e:close()
assert(K.validate(b,{{}},T) and b.counts.changed=='1' and lift_calls==1)
unregistered=true;host.args=args('healthy','/noncanonical.txt')
assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/noncanonical.txt','rb'));local u=K.parse(e:read('*a'));e:close()
assert(u.verdict.status=='UNREACHED' and u.counts.failed=='0' and u.counts.births=='1' and u.counts.moving=='0')
print('normal_session_portable_controls=13 codec_positive=1 reference_faults=5 before_effect_gate=2 empty=1 unrelated_failure=1 full_transport_healthy_B=2 noncanonical_ineligible=1')
"#,
        base = path(&lua),
        positive = path(&positive),
        damaged = path(&damaged),
        root = path(&root)
    );
    let file = root.join("transport.lua");
    std::fs::write(&file, script).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_eech-world")).arg(&file).output().unwrap();
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    println!("{stdout}\n{stderr}");
    assert!(result.status.success(), "{}\n{stdout}\n{stderr}", result.status);
    assert!(stdout.contains("normal_session_portable_controls=13"));
}
