-- Opt-in read-only normal campaign fire. Fixed Lebanon/seed1/100ms/30+120s.
-- No launch/target/angle/readiness/flag/view override. Native authority requires
-- the separately reviewed Root materialization and actual complete receipts.
local a=assert(host.args);local T=assert(loadfile(assert(a.transaction)))()
local K=assert(loadfile(assert(a.contract)))();local N=assert(loadfile(assert(a.collector)))()
K.event=N.event
local mode=a.mode or 'healthy';assert(mode=='healthy' or mode=='terrain_lift','mode')
local identity={schema='3',mode=mode,capture=assert(a.capture),runtime=assert(a.runtime),bundle=assert(a.bundle),driver=assert(a.driver_sha256),collector=assert(a.collector_sha256),transaction=assert(a.transaction_sha256),contract=assert(a.contract_sha256),reference=a.reference_sha256 or 'none',profile='reference-hosted',scenario='lebanon_retail',seed='1',frame_ms='100',warmup='30',settle='120'}
local allowed={root=true,out=true,mode=true,capture=true,runtime=true,bundle=true,driver_sha256=true,collector=true,collector_sha256=true,transaction=true,transaction_sha256=true,contract=true,contract_sha256=true,reference=true,reference_sha256=true}
for k in pairs(a) do assert(allowed[k],'unsupported argument '..k) end
assert(type(a.root)=='string' and type(a.out)=='string','root/out')
for _,k in ipairs({'bundle','driver','collector','transaction','contract'}) do assert(#identity[k]==64 and identity[k]:match('^[0-9a-f]+$'),'artifact hash') end
assert(#identity.runtime==40 and identity.runtime:match('^[0-9a-f]+$'),'runtime source')
local reference,reference_report
if mode=='terrain_lift' then
 local f=assert(io.open(assert(a.reference),'rb'));local text=f:read('*a');assert(f:close())
 local expected={};for k,v in pairs(identity) do expected[k]=v end
 expected.mode='healthy';expected.capture=identity.capture:gsub(':terrain_lift$',':healthy');expected.reference='none'
 reference_report=K.parse(text);reference=K.healthy(reference_report,expected,T) -- FULL gate before output/provider/boot
 assert(identity.reference~='none' and #identity.reference==64,'reference digest')
else assert(a.reference==nil and identity.reference=='none','healthy reference') end
local out=assert(host.create_exclusive(a.out));local raw=assert(host.create_exclusive(a.out..'.events'))
local capture=N.new(identity.capture,100);local engine,failed,frame= nil,nil,0;local booting=true
local registrations,scenes,types={},{},{};local bind_count,changed,moving=0,0,0
local records,moves={},{};local selected,evidence,previous,current;local locked;local awaiting_birth=false;local oracle=K.oracle()
local move_sequence,last_model_call=0,0
local observed_modes={};local mode_codes={move=1,stop=2,place=3,ship_move=4,person_begin=5,person_end=6,fixed_wing_move=7,helicopter_begin=8,helicopter_end=9}
local accounting={};for k in K.schemas.accounting:gmatch('%S+') do accounting[k]=0 end
local outcomes={};for i=1,21 do outcomes[i]=0 end
local function issue(why) failed=failed or tostring(why) end
local function finite(x) return type(x)=='number' and x==x and math.abs(x)<math.huge end
local function vector(v) return type(v)=='table' and #v==3 and finite(v[1]) and finite(v[2]) and finite(v[3]) end
local function csv(v) return string.format('%.17g,%.17g,%.17g',v[1],v[2],v[3]) end
local function model(op,scene,depth,kind,definition,key)
 if failed then error(failed,0) end
 local call=capture:model_entry(op)
 last_model_call=call
 local request,snapshot,observed
 if op=='entity_candidate' then
  request=assert(T.request(scene,depth,kind,definition,key,K.patterns));snapshot=moves[request.identity];observed=observed_modes[request.identity]
  if snapshot and (snapshot.call~=(observed and observed.call) or snapshot.frame>frame or snapshot.after_model>=call) then snapshot=nil end
 end
 local answer=host.model(op,scene,depth,kind,definition,key) -- exactly once, unchanged
 local r={op=op,scene=scene,answer=answer,q=request,move=snapshot,observed_mode=observed and observed.code or 0,observed_call=observed and observed.call or 0,receipt_state=observed and observed.receipt_state or 0,last_receipt_frame=observed and observed.receipt_frame or 0,last_receipt_call=observed and observed.receipt_call or 0}
 local ok,why=pcall(function()
  if op=='scene' then assert(type(answer.name)=='string','scene name');scenes[scene]=answer.name
  elseif op=='entity_candidate' then
   local err,refused=T.answer(r.q,answer,scenes,K.patterns);assert(not err,err)
   if refused then issue('real candidate refusal: '..answer.reason) end
  elseif op=='entity_scan' and type(key)=='string' then r.flag=oracle:check(key,answer) end
 end)
 if not ok then issue(why) end
 -- Boot still delegates and validates scene/store effects exactly once. It
 -- cannot select a weapon query. Retain only answers a frame query can consume.
 if not booting and (r.q or (r.flag and (r.flag.phase==19 or r.flag.phase==21))) then records[call]=r end
 return answer
end
local function move(step)
 if failed then error(failed,0) end
 move_sequence=move_sequence+1;local move_call=move_sequence;local before_model=last_model_call
 assert(move_call<=9007199254740991,'movement callback ordinal overflow')
 if type(step)=='table' and type(step.host_id)=='string' then
  local prior=moves[step.host_id];local previous=observed_modes[step.host_id]
  moves[step.host_id]=nil -- invalidate BEFORE any delegate, including failed/unfinished/unsupported modes
  observed_modes[step.host_id]={code=mode_codes[step.mode] or 10,call=move_call,frame=frame,receipt_state=2,receipt_frame=prior and prior.frame or (previous and previous.receipt_frame or 0),receipt_call=prior and prior.call or (previous and previous.receipt_call or 0)}
 end
 local delegated=step
 if reference and move_call==tonumber(reference.move_call) then
  assert(frame==tonumber(reference.move_frame) and type(step)=='table' and step.host_id==reference.target and step.mode=='move' and before_model==tonumber(reference.move_before),'same actual movement callback before effect')
  K.common(reference_report,capture:prefix(tonumber(reference.w)),frame)
  local r=registrations[step.host_id];assert(r and tostring(r.member)==reference.member and tostring(r.group)==reference.group and tostring(r.bind)==reference.bind,'same canonical registration before effect')
  assert(changed==0 and type(step.terrain)=='table' and finite(step.terrain.elevation) and type(step.map_max)=='table' and finite(step.map_max[2]) and step.terrain.elevation+500<=step.map_max[2],'eligible terrain input')
  assert(string.format('%.17g',step.terrain.elevation)==reference.base and string.format('%.17g',step.map_max[2])==reference.ceiling,'same terrain input')
  delegated={};for k,v in pairs(step) do delegated[k]=v end;delegated.terrain={};for k,v in pairs(step.terrain) do delegated.terrain[k]=v end
  delegated.terrain.elevation=step.terrain.elevation+500;changed=changed+1
 end
 local answer=host.move_vehicle(delegated)
 if type(step)=='table' and step.mode=='move' then
  if type(answer)=='table' and vector(answer.position) and vector(answer.motion_vector) and answer.pending~=true then
   local t=step.terrain;local ceiling=type(step.map_max)=='table' and step.map_max[2] or nil
   if observed_modes[step.host_id].call==move_call then moves[step.host_id]={identity=step.host_id,mode=step.mode,frame=frame,call=move_call,before_model=before_model,after_model=last_model_call,position={answer.position[1],answer.position[2],answer.position[3]},motion={answer.motion_vector[1],answer.motion_vector[2],answer.motion_vector[3]},base=type(t)=='table' and t.elevation or nil,ceiling=ceiling};observed_modes[step.host_id].receipt_state=1;observed_modes[step.host_id].receipt_frame=frame;observed_modes[step.host_id].receipt_call=move_call end
  else issue('invalid real move reply') end
 end;return answer
end
local function choose(query)
 local b,st,f=query.birth,query.step_record,query.normal_fire
 accounting.processed=accounting.processed+1;local first=accounting.processed==1
 if first then
  accounting.first_query=query.query;accounting.first_frame=frame;accounting.first_selector=query.which;accounting.first_models=#query.models;accounting.first_birth_frame=f and f.frame or 0
  for _,event in ipairs(query.models) do local r=records[event.call];if r and r.q then accounting.first_candidates=accounting.first_candidates+1 end end
 end
 local function done(code)
  accounting.decisions=accounting.decisions+1;outcomes[code]=outcomes[code]+1
  if first and accounting.first_code==0 then accounting.first_code=code end
 end
 if not b then return done(1) end
 if not f then return done(2) end
 if b.launcher=='none' then return done(3) end
 if f.frame<=300 then return done(4) end
 if not st.same_launcher then return done(5) end
 if not registrations[b.launcher] then return done(6) end
 if locked and (b.fire~=locked.fire or b.detach~=locked.detach or b.w~=locked.w or b.launcher~=locked.launcher or b.weapon~=locked.weapon) then return done(7) end
 local seen=false
 for i,event in ipairs(query.models) do
  local r=records[event.call] or {}
  if event.op==16 then assert(r.q,'missing live candidate entry') end
  if r.q and T.eligible(r.q) and r.answer.result=='need_updated' then
   seen=true;local id=r.q.identity;local m,registration=r.move,registrations[id]
   if first then
    accounting.first_need=accounting.first_need+1
    if accounting.first_need==1 then accounting.first_mode=r.observed_mode;accounting.first_registered=registration and 1 or 0;accounting.first_type=types[id] or 0;accounting.first_move_frame=r.last_receipt_frame;accounting.first_move_call=r.last_receipt_call;accounting.first_latest_call=r.observed_call;accounting.first_later_call=0;accounting.first_receipt=m and (m.frame<=f.frame and 4 or 1) or r.receipt_state;local later=observed_modes[id];if not m and r.receipt_state==0 and later and later.call>r.observed_call then accounting.first_receipt=5;accounting.first_later_call=later.call end end
   end
   local reject
   if id==b.launcher then reject=19
   elseif locked and (id~=locked.target or r.q.scene~=locked.scene) then reject=15
   elseif not id:match('^eech%-world:[1-9]%d*$') or not registration then reject=10
   elseif not m then reject=9
   elseif not finite(m.base) then reject=11
   elseif not finite(m.ceiling) then reject=12
   elseif m.base+500>m.ceiling then reject=13
   elseif m.motion[1]^2+m.motion[2]^2+m.motion[3]^2<=1 then reject=14 end
   if reject then done(reject)
   else
    for axis=1,3 do
     if K.number(r.q.words[13+axis])~=m.position[axis] or K.number(r.q.words[16+axis])~=m.motion[axis] then done(9);error('existing candidate move does not match request',0) end
    end
    local j=i+1;local next_r=query.models[j] and records[query.models[j].call]
    if next_r and next_r.flag and next_r.flag.phase==19 then j=j+1;next_r=query.models[j] and records[query.models[j].call] end
    if not next_r or not next_r.flag or next_r.flag.phase~=21 or next_r.flag.array~=0 then done(16)
    else
     local read=next_r.flag;local carry_event=query.models[j+1];local carry=carry_event and records[carry_event.call]
     if first then accounting.first_reads=accounting.first_reads+1 end
     if not carry or not carry.q or T.same_reask({scene=r.q.scene,identity=id,words=r.q.words,answered_word=read.word},carry.q) then done(17)
     else
      if first then accounting.first_reasks=accounting.first_reasks+1 end
      local test_event,test
      for k=j+1,#query.models do
       local candidate=query.models[k];local reply=records[candidate.call]
       if reply and reply.q and reply.q.identity==id and reply.q.scene==r.q.scene and reply.answer.result=='test' and K.patterns(reply.answer.p,16) then test_event,test=candidate,reply;break end
      end
      if not test then done(18)
      else
       if K.number(K.patterns(test.answer.p,16)[16])<=1 then done(18);error('live candidate TEST is not moving',0) end
       if first then accounting.first_tests=accounting.first_tests+1 end
       local s={query_frame=frame,move_frame=m.frame,birth_frame=f.frame,fire=b.fire,detach=b.detach,w=b.w,step=query.step,query=query.query,before=query.before,after=query.result.after,need=event.call,read=query.models[j].call,carry=carry_event.call,test=test_event.call,launcher=b.launcher,birth_target=b.target,step_target=st.target,end_target=st.result.target,target=id,move_call=m.call,move_before=m.before_model,move_after=m.after_model,weapon=b.weapon,scene=r.q.scene,name=T.hex(r.answer.name),member=registration.member,group=registration.group,bind=registration.bind,key=read.key,lane=read.lane,word=read.word,fact=carry.q.words[21],pose=test.answer.p,request=table.concat(r.q.words,','),reask=table.concat(carry.q.words,','),position=csv(m.position),motion=csv(m.motion),base=string.format('%.17g',m.base),ceiling=string.format('%.17g',m.ceiling),previous=string.format('%.17g',previous),current=string.format('%.17g',current)}
       local ce={call=event.call,scene=r.q.scene,identity=id,member=registration.member,group=registration.group,bind=registration.bind,move_identity=m.identity,move_call=m.call,move_before=m.before_model,move_after=m.after_model,mode=m.mode,move_frame=m.frame,query_frame=frame,name=s.name,read_call=s.read,read_key=read.key,lane=read.lane,word=read.word,carry_call=carry_event.call,carry_identity=carry.q.identity,carry_scene=carry.q.scene,request=s.request,reask=s.reask,test_call=test_event.call,test_identity=test.q.identity,test_scene=test.q.scene,test_request=table.concat(test.q.words,','),pose=s.pose}
       local ev={fire=T.hex(f.literal),detach=T.hex(b.detachment.literal),birth=T.hex(b.literal),step=T.hex(st.literal),query=T.hex(query.literal),query_end=T.hex(query.result.literal),step_end=T.hex(st.result.literal),candidate=T.hex(K.candidate_row(ce))}
       local lines={};for _,v in ipairs(query.models) do lines[#lines+1]=v.literal end;ev.models=T.hex(table.concat(lines,'\n')..'\n')
       if not locked then locked={fire=b.fire,detach=b.detach,w=b.w,launcher=b.launcher,weapon=b.weapon,target=id,scene=r.q.scene} end
       if f.frame>=m.frame then awaiting_birth=true;return done(20) end
       if not selected then selected,evidence=s,ev end;moving=moving+1;return done(21)
      end
     end
    end
   end
  end
 end
 if not seen then done(8) end
end
local ok,why=xpcall(function()
 local dc=require('eech_dc');local root=a.root
 dc.prepare_installation(root)
 host.use_reference_arithmetic ()
 engine=dc.boot{install_root=root,map='..\\common\\maps\\map5',campaign_directory='camp01',campaign='LEBANON.CHC',gunship='apache',seed=1,terrain=host.terrain_provider(root,'..\\common\\maps\\map5'),environment=host.env_provider(),bind_mobile=function(d)
  for id,r in pairs(registrations) do if type(d)=='table' and r.member==d.member_key and r.group==d.group_key then local prior=moves[id];local previous=observed_modes[id];moves[id]=nil;observed_modes[id]={code=11,call=previous and previous.call or 0,frame=frame,receipt_state=3,receipt_frame=prior and prior.frame or (previous and previous.receipt_frame or 0),receipt_call=prior and prior.call or (previous and previous.receipt_call or 0)} end end
  local id=host.bind_mobile(d);moves[id]=nil;observed_modes[id]=nil;bind_count=bind_count+1;assert(T.registration(registrations,d,id,bind_count));types[id]=type(d.entity_type)=='number' and d.entity_type>=0 and d.entity_type==math.floor(d.entity_type) and d.entity_type or 0;return id
 end,move_vehicle=move,observe_loss=host.observe_loss,supply_reached=host.supply_reached,line_of_sight=host.line_of_sight,model=model,sense=host.sense,aim=host.aim}
 if failed then error(failed,0) end
 local clock=engine:clock();assert(clock.elapsed_seconds==0,'fresh seed1 clock')
 local boot=engine:wpn_events();assert(raw:write(boot));local boot_queries=capture:feed(1,0,boot)
 assert(#boot_queries==0,'unsupported weapon query during boot') -- diagnose, never silently discard a query
 capture:drained();booting=false;records={} -- boot receipts are observable provenance, never post-birth effect credit
 for n=1,1500 do
  frame=n;previous=clock.elapsed_seconds;engine:frame(100);if failed then error(failed,0) end;clock=engine:clock();current=clock.elapsed_seconds
  assert(finite(previous) and finite(current) and current>previous,'real frame clock')
  local events=engine:wpn_events();assert(raw:write(events))
  local queries=capture:feed(frame,(frame-1)*100,events)
  for _,q in ipairs(queries) do choose(q) end
  capture:drained()
  records={} -- canonical receipt persists until an actual newer writer, not a frame clock
 end
 -- The actual session userdata has no close/stop/finalize method. Retain its
 -- handle until script completion, as campaign.lua does; no extra native entry.
end,function(e)return tostring(e)end)
if not ok then issue(why) end
local finished;local clean,err=pcall(function()finished=capture:finish()end);if not clean then issue(err) end
local counts={frames=frame,fires=finished and #finished.fires or 0,births=finished and finished.births or 0,steps=finished and #finished.steps or 0,queries=finished and #finished.queries or 0,model_calls=finished and finished.model_calls or 0,moving=moving,failed=failed and 1 or 0,changed=changed}
accounting.outcomes=table.concat(outcomes,',')
if reference and not selected and not failed then issue('changed case did not retain selected cohort');counts.failed=1 end
local status=failed and 'FAIL' or (selected and 'PASS' or 'UNREACHED')
if reference and status=='PASS' then
 for _,k in ipairs({'query_frame','move_frame','move_call','move_before','birth_frame','fire','detach','w','step','query','launcher','target','weapon','scene','member','group','bind'}) do if tostring(selected[k])~=reference[k] then issue('changed selected identity '..k) end end
 if changed~=1 or selected.pose==reference.pose or selected.position==reference.position then issue('terrain input has no same-cohort pose effect') end
 if failed then counts.failed=1;status='FAIL' end
end
if status=='PASS' then
 local text=K.row('identity',identity)..'\n'..K.row('counts',counts)..'\n'..K.row('accounting',accounting)..'\n'..K.row('selection',selected)..'\n'..K.row('evidence',evidence)..'\n'..K.row('verdict',{status=status,reason=T.hex(mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect')})..'\n'
 local valid,problem=pcall(function()return K.validate(K.parse(text),{},T)end)
 if valid and reference then valid,problem=pcall(function()return K.paired(K.parse(text),reference_report)end) end
 if not valid then issue(problem);counts.failed=1;status='FAIL' end
end
assert(out:write(K.row('identity',identity)..'\n'..K.row('counts',counts)..'\n'))
assert(out:write(K.row('accounting',accounting)..'\n'))
if selected then assert(out:write(K.row('selection',selected)..'\n'..K.row('evidence',evidence)..'\n')) end
assert(out:write(K.row('verdict',{status=status,reason=T.hex(failed or (selected and (mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect') or (awaiting_birth and 'locked cohort lacks earlier observed eligible query' or 'no eligible normal moving cohort')))})..'\n'));assert(raw:close());assert(out:close())
assert(status=='PASS','normal-fire-session: '..status..': '..tostring(failed or 'fixed window has no eligible cohort'))
