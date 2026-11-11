
-- REAL reference-host bind/movement/SharedMovers/model/FlagStore collaborators.
-- Only Engine frame/event transport is synthetic; no retail/normal-fire claim.
local K=assert(loadfile(base..'/normal-fire-session-contract.lua'))()
local N=assert(loadfile(base..'/normal-fire-correlation.lua'))();K.event=N.event
local T=assert(loadfile(base..'/physical-flags-read-transaction.lua'))()
local actual_bind,actual_move=host.bind_mobile,host.move_vehicle
local input=assert(io.open(positive,'rb'));local baseline=K.parse(input:read('*a'));input:close()
local Z,O='0x00000000','0x3f800000'
local function flag_request(phase)
 local w={};for i=1,16 do w[i]=Z end;w[12]=K.whole(phase)
 if phase==19 then w[1]=O else w[14]=O end
 return table.concat(w,',')..(phase==21 and ' f1' or '')
end
-- Schema4 cases. Ineligible healthy MOVEs reach the REAL solver and stay
-- UNREACHED by code; B_* deliver one different consumed field at the exact
-- reference callback and must refuse BEFORE the zero-step effect.
local INELIGIBLE={terrain='22',zero_dt='24',reverse='25',outside='26',waypoint='27'}
local B_REFUSAL={B_dt=true,B_input=true,B_terrain=true,B_bounds=true}
local B_MODE=CASE=='B' or CASE=='B_writer' or B_REFUSAL[CASE]
local function args(mode,out,ref)
 return {root=root,out=root..out,mode=mode,capture='fixture:'..mode,runtime=string.rep('b',40),bundle=string.rep('a',64),driver_sha256=string.rep('a',64),collector=base..'/normal-fire-correlation.lua',collector_sha256=string.rep('a',64),transaction=base..'/physical-flags-read-transaction.lua',transaction_sha256=string.rep('a',64),contract=base..'/normal-fire-session-contract.lua',contract_sha256=string.rep('a',64),reference=ref,reference_sha256=ref and string.rep('c',64) or nil}
end
local persistent_provider=host.model_provider(host.model_store(modelroot))
-- Exact transport of this fixture's returned f32 values (including signed0).
-- No narrowing/guessed integer physical inputs; these values came from real MOVE.
local function f32(value)
 assert(type(value)=='number' and value==value and math.abs(value)<math.huge)
 local sign=(value<0 or (value==0 and 1/value<0)) and 2147483648 or 0
 if value==0 then return string.format('0x%08x',sign) end
 local mantissa,exponent=math.frexp(math.abs(value));exponent=exponent-1
 assert(exponent>=-126 and exponent<=127,'fixture returned non-normal f32')
 local fraction=(mantissa*2-1)*8388608;assert(fraction==math.floor(fraction),'fixture value is not f32')
 return string.format('0x%08x',sign+(exponent+127)*8388608+fraction)
end
local completed_move
-- Every delegate the driver makes reaches the REAL host through this counter;
-- the engine's own step is kept to prove that ONLY delta_seconds was cloned.
local delegated,engine_step={}
local function counted_move(step) delegated[#delegated+1]={step=step,delta=step.delta_seconds,engine=engine_step};return actual_move(step) end
local function persistent_run(label,reference_path)
 host.bind_mobile=actual_bind;host.move_vehicle=counted_move;host.model=persistent_provider
 package.loaded.eech_dc=nil
 package.preload.eech_dc=function()return {prepare_installation=function()end,boot=function(o)
  local I={1,0,0,0,1,0,0,0,1};local p={position={10,100,20},velocity=CASE=='reverse' and -3 or 2,attitude=I}
  local function bind(member,pose)
   return o.bind_mobile{member_key=member,group_key=800,entity_id=member,entity_type=23,sub_type=1,side='blue',pose=pose or p,movement_kind=1,motion_vector={0,0,2}}
  end
  local launcher=bind(8001,{position={10,100,20},velocity=2,attitude=I});local candidate=bind(8002)
  o.model('scene',1);local opened=o.model('entity_scan',0,nil,nil,nil,flag_request(19));local key=opened.name
  for _=1,8 do o.model('scene',1) end -- ten real provider entries before query
  local count,pending=0,'';local result={position=p.position,velocity=p.velocity,attitude=p.attitude,motion_vector={0,0,2}}
  local function fields(row)
   local words={};for i,value in ipairs(row) do words[i]=f32(value) end;return table.concat(words,',')
  end
  local function prefix()
   local e=baseline.evidence
   return (K.unhex(e.fire)..'\n'..K.unhex(e.detach)..'\n'..K.unhex(e.birth)..'\nnormal-ev seq=4 tick=30000 kind=normal_end fire=1 births=1\n'):gsub('eech%-world:1',launcher):gsub('eech%-world:2',candidate)
  end
  local function advance(reference_callback)
   -- Terrain absent, as rv_hostmv.c sends when the guide does not follow
   -- terrain. Each named case changes ONE consumed field.
   local step={host_id=candidate,mode='move',pose=p,delta_seconds=CASE=='zero_dt' and 0 or 1,desired_velocity=2,max_acceleration=1,waypoint=CASE=='waypoint' and {10.5,100,20.5} or {10,100,100},tacview_logging=false,map_min={0,-1000,CASE=='outside' and 30 or 0},map_max={4095,10000,4095}}
   if CASE=='terrain' then step.terrain={elevation=100,forest=false,face_normal={0,1,0}} end
   if reference_callback then
    if CASE=='B_dt' then step.delta_seconds=0.5 elseif CASE=='B_input' then step.desired_velocity=3
    elseif CASE=='B_terrain' then step.terrain={elevation=100,forest=false,face_normal={0,1,0}} elseif CASE=='B_bounds' then step.map_max={4095,10000,4000} end
   end
   engine_step=step;local before=step.delta_seconds;result=o.move_vehicle(step);assert(step.delta_seconds==before,'caller input mutated')
   assert(result.motion_vector[1]^2+result.motion_vector[2]^2+result.motion_vector[3]^2>1)
   completed_move={position=result.position,velocity=result.velocity,attitude=result.attitude};p=completed_move
  end
  if CASE=='boot' then advance() end
  return {clock=function()return {elapsed_seconds=count/10}end,
   frame=function(_,ms)
    assert(ms==100);count=count+1
    if count==301 then pending=prefix() end
    if (CASE=='prebirth' and count==300) or (CASE~='prebirth' and CASE~='boot' and CASE~='future' and count==302) then advance(true);if CASE=='repeat' then advance() end end
    if count==303 then
     if CASE=='bind' then bind(8002)
     elseif CASE=='unrelated' then o.move_vehicle{host_id=launcher,mode='stop',pose={position={10,100,20},velocity=2,attitude=I}}
     elseif CASE=='stop' or CASE=='place' or CASE=='routed_vector' or CASE=='retire' or CASE=='invalid' or CASE=='B_writer' then
      local step={host_id=candidate,mode=CASE=='B_writer' and 'stop' or CASE,pose=p,motion_vector_in=result.motion_vector}
      if CASE=='place' then step.motion_vector_in=nil;step.motion_vector=result.motion_vector end
      local okay,why=pcall(o.move_vehicle,step)
      if CASE=='invalid' then assert(not okay and tostring(why):find('unknown movement mode',1,true));error(why,0) end
     end
     local words={};for i=1,44 do words[i]=Z end
     words[1]=f32(10);words[2]=Z;words[3]=f32(20)
     words[4]=f32(10);words[5]=f32(1000);words[6]=f32(24);words[7]=O;words[20]='0x3dcccccd'
     for i=1,3 do words[13+i]=f32(result.position[i]);words[16+i]=f32(result.motion_vector[i]) end
     local q=table.concat(words,',')..' '..(CASE=='wrong' and 'eech-world:99999' or candidate)
     assert(o.model('entity_candidate',1,nil,nil,nil,q).result=='need_updated')
     local read=o.model('entity_scan',0,nil,nil,nil,flag_request(21):gsub(' f[^ ]+$',' '..key))
     assert(read.result=='flag_read');local raw=assert(K.patterns(read.p,3));assert(raw[3]==Z)
     words[21]=O;assert(o.model('entity_candidate',1,nil,nil,nil,table.concat(words,',')..' '..candidate).result=='need_placement')
     local pose={};for _,v in ipairs(result.position) do pose[#pose+1]=v end;for _,v in ipairs(result.motion_vector) do pose[#pose+1]=v end;for _,v in ipairs(result.attitude) do pose[#pose+1]=v end
     assert(o.model('entity_pose',0,1,23,1,fields(pose)..' '..candidate).result=='recorded')
     words[44]=O;for i=1,9 do words[27+i]=f32(result.attitude[i]) end
     local answer=o.model('entity_candidate',1,nil,nil,nil,table.concat(words,',')..' '..candidate)
     assert(answer.result=='test' and #assert(K.patterns(answer.p,16))==16);print('REAL_NEED_READ_REASK_TEST=PASS move_frame302 query_frame303 case='..CASE)
     if CASE=='future' then advance() end
     local e=baseline.evidence
     pending=(K.unhex(e.step)..'\n'..K.unhex(e.query)..'\n'..K.unhex(e.models)..K.unhex(e.query_end)..'\n'..K.unhex(e.step_end)..'\n'):gsub('tick=30100','tick=30200'):gsub('eech%-world:1',launcher):gsub('eech%-world:2',candidate)
    end
   end,wpn_events=function()local value=pending;pending='';return value end}
 end}end
 host.args=args(reference_path and 'zero_step' or 'healthy','/persistent-'..label..'.txt',reference_path)
 local accepted,why=pcall(assert(loadfile(base..'/normal-fire-session.lua')))
 local f=assert(io.open(root..'/persistent-'..label..'.txt','rb'));local report=K.parse(f:read('*a'));f:close()
 local a=report.accounting
 if CASE=='healthy' or CASE=='B' or CASE=='unrelated' or CASE=='repeat' then
  assert(accepted and report.verdict.status=='PASS','real retained-state consumer must qualify: '..tostring(why)..' '..report.verdict.status..' first_code='..a.first_code..' first_receipt='..a.first_receipt..' outcomes='..a.outcomes)
 elseif B_REFUSAL[CASE] then
  -- Refused at the exact callback BEFORE any delegate: the real host saw nothing.
  assert(not accepted and report.verdict.status=='FAIL' and report.counts.changed=='0' and report.counts.moving=='0' and #delegated==0,tostring(why))
  assert(K.unhex(report.verdict.reason):find('same consumed MOVE input before effect',1,true),K.unhex(report.verdict.reason))
 elseif CASE=='B_writer' then
  -- A newer same-ID writer after the single zero-step effect removes its credit.
  assert(not accepted and report.verdict.status=='FAIL' and report.counts.changed=='1' and report.counts.moving=='0' and a.first_code=='9' and a.first_receipt=='2',tostring(why))
  assert(K.unhex(report.verdict.reason)=='changed case did not retain selected cohort',K.unhex(report.verdict.reason))
 else
  assert(not accepted and report.counts.changed=='0' and report.counts.moving=='0',tostring(why))
  if CASE=='retire' or CASE=='invalid' or CASE=='wrong' then assert(report.verdict.status=='FAIL')
  elseif CASE=='prebirth' or CASE=='boot' then assert(report.verdict.status=='UNREACHED' and a.first_code=='20' and a.first_receipt=='4')
  elseif INELIGIBLE[CASE] then assert(report.verdict.status=='UNREACHED' and a.first_code==INELIGIBLE[CASE] and a.first_receipt=='1','ineligible '..CASE..' code '..a.first_code)
  else local reason=({bind='3',future='5'})[CASE] or '2';assert(report.verdict.status=='UNREACHED' and a.first_code=='9' and a.first_receipt==reason,'receipt '..tostring(a.first_receipt)..' '..reason) end
 end
 return report
end
local r=persistent_run(CASE,B_MODE and reference or nil)
if CASE=='healthy' or CASE=='B' or CASE=='unrelated' or CASE=='repeat' then
 assert(r.selection.query_frame=='303' and r.selection.move_frame=='302' and r.selection.terrain=='absent')
 if CASE=='B' then
  local f=assert(io.open(reference,'rb'));local h=K.parse(f:read('*a'));f:close();assert(K.paired(r,h) and r.counts.changed=='1')
  -- ONE delegate with step time 0; every other consumed field is the engine's.
  local zero=0
  for _,call in ipairs(delegated) do
   if call.delta==0 then
    zero=zero+1;assert(call.step~=call.engine and call.engine.delta_seconds==1)
    for k,v in pairs(call.engine) do assert(k=='delta_seconds' or call.step[k]==v,'cloned field '..k) end
    for k in pairs(call.step) do assert(call.engine[k]~=nil,'extra field '..k) end
   else assert(call.step==call.engine) end
  end
  assert(zero==1 and r.selection.position==string.format('%.17g,%.17g,%.17g',10,100,20) and h.selection.position~=r.selection.position,'genuine zero-step result at the pre-position')
  print('ZERO_STEP_PAIR=PASS healthy_position='..h.selection.position..' zero_step_position='..r.selection.position..' delegated='..r.selection.delegated..' prestate='..r.selection.prestate)
 end
 if CASE=='repeat' then assert(r.selection.move_call=='2') end
end
print('persistent_real_collaborators=PASS case='..CASE)
