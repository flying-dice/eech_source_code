
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
local function persistent_run(label,reference_path)
 host.bind_mobile=actual_bind;host.move_vehicle=actual_move;host.model=persistent_provider
 package.loaded.eech_dc=nil
 package.preload.eech_dc=function()return {prepare_installation=function()end,boot=function(o)
  local I={1,0,0,0,1,0,0,0,1};local p={position={10,100,20},velocity=2,attitude=I}
  local function bind(member)
   return o.bind_mobile{member_key=member,group_key=800,entity_id=member,entity_type=23,sub_type=1,side='blue',pose=p,movement_kind=1,motion_vector={0,0,2}}
  end
  local launcher=bind(8001);local candidate=bind(8002)
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
  local function advance()
   local step={host_id=candidate,mode='move',pose=p,delta_seconds=1,desired_velocity=2,max_acceleration=1,waypoint={10,100,100},tacview_logging=false,terrain={elevation=100,forest=false,face_normal={0,1,0}},map_min={0,-1000,0},map_max={4095,10000,4095}}
   result=o.move_vehicle(step);assert(step.terrain.elevation==100)
   assert(result.motion_vector[1]^2+result.motion_vector[2]^2+result.motion_vector[3]^2>1)
   completed_move={position=result.position,velocity=result.velocity,attitude=result.attitude};p=completed_move
  end
  if CASE=='boot' then advance() end
  return {clock=function()return {elapsed_seconds=count/10}end,
   frame=function(_,ms)
    assert(ms==100);count=count+1
    if count==301 then pending=prefix() end
    if (CASE=='prebirth' and count==300) or (CASE~='prebirth' and CASE~='boot' and CASE~='future' and count==302) then advance();if CASE=='repeat' then advance() end end
    if count==303 then
     if CASE=='bind' then bind(8002)
     elseif CASE=='unrelated' then o.move_vehicle{host_id=launcher,mode='stop',pose={position={10,100,20},velocity=2,attitude=I}}
     elseif CASE~='healthy' and CASE~='B' and CASE~='prebirth' and CASE~='boot' and CASE~='future' and CASE~='wrong' and CASE~='repeat' then
      local step={host_id=candidate,mode=CASE,pose=p,motion_vector_in=result.motion_vector}
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
 host.args=args(reference_path and 'terrain_lift' or 'healthy','/persistent-'..label..'.txt',reference_path)
 local accepted,why=pcall(assert(loadfile(base..'/normal-fire-session.lua')))
 local f=assert(io.open(root..'/persistent-'..label..'.txt','rb'));local report=K.parse(f:read('*a'));f:close()
 if CASE=='healthy' or CASE=='B' or CASE=='unrelated' or CASE=='repeat' then
  assert(accepted and report.verdict.status=='PASS','real retained-state consumer must qualify: '..tostring(why)..' '..report.verdict.status)
 else
  assert(not accepted and report.counts.changed=='0' and report.counts.moving=='0',tostring(why))
  if CASE=='retire' or CASE=='invalid' or CASE=='wrong' then assert(report.verdict.status=='FAIL')
  elseif CASE=='prebirth' or CASE=='boot' then assert(report.verdict.status=='UNREACHED' and report.accounting.first_code=='20' and report.accounting.first_receipt=='4')
  else local reason=({bind='3',future='5'})[CASE] or '2';assert(report.verdict.status=='UNREACHED' and report.accounting.first_code=='9' and report.accounting.first_receipt==reason,'receipt '..tostring(report.accounting.first_receipt)..' '..reason) end
 end
 return report
end
local r=persistent_run(CASE,CASE=='B' and reference or nil)
if CASE=='healthy' or CASE=='B' or CASE=='unrelated' or CASE=='repeat' then
 assert(r.selection.query_frame=='303' and r.selection.move_frame=='302')
 if CASE=='B' then local f=assert(io.open(reference,'rb'));local h=K.parse(f:read('*a'));f:close();assert(K.paired(r,h) and r.counts.changed=='1') end
 if CASE=='repeat' then assert(r.selection.move_call=='2') end
end
print('persistent_real_collaborators=PASS case='..CASE)
