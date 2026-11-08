-- Opt-in read-only normal campaign fire. Fixed Lebanon/seed1/100ms/30+120s.
-- No launch/target/angle/readiness/flag/view override. Native authority requires
-- the separately reviewed Root materialization and actual complete receipts.
local a=assert(host.args);local T=assert(loadfile(assert(a.transaction)))()
local K=assert(loadfile(assert(a.contract)))();local N=assert(loadfile(assert(a.collector)))()
K.event=N.event
local mode=a.mode or 'healthy';assert(mode=='healthy' or mode=='terrain_lift','mode')
local identity={schema='1',mode=mode,capture=assert(a.capture),runtime=assert(a.runtime),bundle=assert(a.bundle),driver=assert(a.driver_sha256),collector=assert(a.collector_sha256),transaction=assert(a.transaction_sha256),contract=assert(a.contract_sha256),reference=a.reference_sha256 or 'none',profile='reference-hosted',scenario='lebanon_retail',seed='1',frame_ms='100',warmup='30',settle='120'}
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
local registrations,scenes={},{};local bind_count,changed,moving=0,0,0
local records,moves={},{};local selected,evidence,previous,current;local locked;local awaiting_birth=false;local oracle=K.oracle()
local function issue(why) failed=failed or tostring(why) end
local function finite(x) return type(x)=='number' and x==x and math.abs(x)<math.huge end
local function vector(v) return type(v)=='table' and #v==3 and finite(v[1]) and finite(v[2]) and finite(v[3]) end
local function csv(v) return string.format('%.17g,%.17g,%.17g',v[1],v[2],v[3]) end
local function model(op,scene,depth,kind,definition,key)
 if failed then error(failed,0) end
 local call=capture:model_entry(op)
 local answer=host.model(op,scene,depth,kind,definition,key) -- exactly once, unchanged
 local r={op=op,scene=scene,answer=answer}
 local ok,why=pcall(function()
  if op=='scene' then assert(type(answer.name)=='string','scene name');scenes[scene]=answer.name
  elseif op=='entity_candidate' then
   r.q=assert(T.request(scene,depth,kind,definition,key,K.patterns))
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
 local delegated=step
 if reference and frame==tonumber(reference.frame) and type(step)=='table' and step.host_id==reference.target and step.mode=='move' then
  K.common(reference_report,capture:prefix(tonumber(reference.w)),frame)
  local r=registrations[step.host_id];assert(r and tostring(r.member)==reference.member and tostring(r.group)==reference.group and tostring(r.bind)==reference.bind,'same canonical registration before effect')
  assert(changed==0 and type(step.terrain)=='table' and finite(step.terrain.elevation) and type(step.map_max)=='table' and finite(step.map_max[2]) and step.terrain.elevation+500<=step.map_max[2],'eligible terrain input')
  assert(string.format('%.17g',step.terrain.elevation)==reference.base and string.format('%.17g',step.map_max[2])==reference.ceiling,'same terrain input')
  delegated={};for k,v in pairs(step) do delegated[k]=v end;delegated.terrain={};for k,v in pairs(step.terrain) do delegated.terrain[k]=v end
  delegated.terrain.elevation=step.terrain.elevation+500;changed=changed+1
 end
 local answer=host.move_vehicle(delegated)
 if type(step)=='table' and step.mode=='move' then
  if type(answer)=='table' and vector(answer.position) and vector(answer.motion_vector) then
   local t=step.terrain;local ceiling=type(step.map_max)=='table' and step.map_max[2] or nil
   moves[step.host_id]={position=answer.position,motion=answer.motion_vector,base=type(t)=='table' and t.elevation or nil,ceiling=ceiling}
  else issue('invalid real move reply') end
 end;return answer
end
local function choose(query)
 local b,st,f=query.birth,query.step_record,query.normal_fire
 if not b or not f or not f.canonical or f.frame<=300 or not st.same_target or not st.same_launcher or not registrations[b.launcher] or st.result.target~=b.target then return end
 local m=moves[b.target];local registration=registrations[b.target]
 if not m or not registration or not finite(m.base) or not finite(m.ceiling) or m.base+500>m.ceiling or m.motion[1]^2+m.motion[2]^2+m.motion[3]^2<=1 then return end
 for i,event in ipairs(query.models) do
  local r=records[event.call] or {}
  if event.op==16 then assert(r.q,'missing live candidate entry') end
  if r.q and r.q.identity==b.target and T.eligible(r.q) and r.answer.result=='need_updated' then
   local j=i+1;local next_r=query.models[j] and records[query.models[j].call]
   if next_r and next_r.flag and next_r.flag.phase==19 then j=j+1;next_r=query.models[j] and records[query.models[j].call] end
   if next_r and next_r.flag and next_r.flag.phase==21 and next_r.flag.array==0 then
    local read=next_r.flag;local carry_event=query.models[j+1];local carry=carry_event and records[carry_event.call]
    if carry and carry.q and not T.same_reask({scene=r.q.scene,identity=b.target,words=r.q.words,answered_word=read.word},carry.q) then
     for k=j+1,#query.models do
      local test_event=query.models[k];local test=records[test_event.call]
      if test and test.q and test.q.identity==b.target and test.q.scene==r.q.scene and test.answer.result=='test' and K.patterns(test.answer.p,16) then
       local s={frame=frame,birth_frame=f.frame,fire=b.fire,detach=b.detach,w=b.w,step=query.step,query=query.query,before=query.before,after=query.result.after,need=event.call,read=query.models[j].call,carry=carry_event.call,test=test_event.call,launcher=b.launcher,target=b.target,weapon=b.weapon,scene=r.q.scene,name=T.hex(r.answer.name),member=registration.member,group=registration.group,bind=registration.bind,key=read.key,lane=read.lane,word=read.word,fact=carry.q.words[21],pose=test.answer.p,request=table.concat(r.q.words,','),reask=table.concat(carry.q.words,','),position=csv(m.position),motion=csv(m.motion),base=string.format('%.17g',m.base),ceiling=string.format('%.17g',m.ceiling),previous=string.format('%.17g',previous),current=string.format('%.17g',current)}
       local ev={fire=T.hex(f.literal),detach=T.hex(b.detachment.literal),birth=T.hex(b.literal),step=T.hex(st.literal),query=T.hex(query.literal),query_end=T.hex(query.result.literal),step_end=T.hex(st.result.literal)}
       local lines={};for _,v in ipairs(query.models) do lines[#lines+1]=v.literal end;ev.models=T.hex(table.concat(lines,'\n')..'\n')
       if not locked then locked={fire=b.fire,detach=b.detach,w=b.w,launcher=b.launcher,target=b.target,weapon=b.weapon} end
       for k,v in pairs(locked) do if b[k]~=v then return end end -- no alternative cohort fallback
       if f.frame>=frame then awaiting_birth=true;return end -- wait only for this SAME cohort, within the fixed window
       if not selected then selected,evidence=s,ev end;moving=moving+1;return
      end
     end
    end
   end
  end
 end
end
local ok,why=xpcall(function()
 local dc=require('eech_dc');local root=a.root
 dc.prepare_installation(root)
 host.use_reference_arithmetic ()
 engine=dc.boot{install_root=root,map='..\\common\\maps\\map5',campaign_directory='camp01',campaign='LEBANON.CHC',gunship='apache',seed=1,terrain=host.terrain_provider(root,'..\\common\\maps\\map5'),environment=host.env_provider(),bind_mobile=function(d)
  local id=host.bind_mobile(d);bind_count=bind_count+1;assert(T.registration(registrations,d,id,bind_count));return id
 end,move_vehicle=move,observe_loss=host.observe_loss,supply_reached=host.supply_reached,line_of_sight=host.line_of_sight,model=model,sense=host.sense,aim=host.aim}
 if failed then error(failed,0) end
 local clock=engine:clock();assert(clock.elapsed_seconds==0,'fresh seed1 clock')
 local boot=engine:wpn_events();assert(raw:write(boot));local boot_queries=capture:feed(1,0,boot)
 assert(#boot_queries==0,'unsupported weapon query during boot') -- diagnose, never silently discard a query
 capture:drained();booting=false;records={};moves={}
 for n=1,1500 do
  frame=n;previous=clock.elapsed_seconds;engine:frame(100);if failed then error(failed,0) end;clock=engine:clock();current=clock.elapsed_seconds
  assert(finite(previous) and finite(current) and current>previous,'real frame clock')
  local events=engine:wpn_events();assert(raw:write(events))
  local queries=capture:feed(frame,(frame-1)*100,events)
  for _,q in ipairs(queries) do choose(q) end
  capture:drained()
  records={};moves={}
 end
 if not failed then engine:close();engine=nil end -- no getter/close after failure
end,function(e)return tostring(e)end)
if not ok then issue(why) end
local finished;local clean,err=pcall(function()finished=capture:finish()end);if not clean then issue(err) end
local counts={frames=frame,fires=finished and #finished.fires or 0,births=finished and finished.births or 0,steps=finished and #finished.steps or 0,queries=finished and #finished.queries or 0,model_calls=finished and finished.model_calls or 0,moving=moving,failed=failed and 1 or 0,changed=changed}
if reference and not selected and not failed then issue('changed case did not retain selected cohort');counts.failed=1 end
local status=failed and 'FAIL' or (selected and 'PASS' or 'UNREACHED')
if reference and status=='PASS' then
 for _,k in ipairs({'frame','birth_frame','fire','detach','w','step','query','launcher','target','weapon','scene','member','group','bind'}) do if tostring(selected[k])~=reference[k] then issue('changed selected identity '..k) end end
 if changed~=1 or selected.pose==reference.pose or selected.position==reference.position then issue('terrain input has no same-cohort pose effect') end
 if failed then counts.failed=1;status='FAIL' end
end
if status=='PASS' then
 local text=K.row('identity',identity)..'\n'..K.row('counts',counts)..'\n'..K.row('selection',selected)..'\n'..K.row('evidence',evidence)..'\n'..K.row('verdict',{status=status,reason=T.hex(mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect')})..'\n'
 local valid,problem=pcall(function()return K.validate(K.parse(text),{},T)end)
 if valid and reference then valid,problem=pcall(function()return K.paired(K.parse(text),reference_report)end) end
 if not valid then issue(problem);counts.failed=1;status='FAIL' end
end
assert(out:write(K.row('identity',identity)..'\n'..K.row('counts',counts)..'\n'))
if selected then assert(out:write(K.row('selection',selected)..'\n'..K.row('evidence',evidence)..'\n')) end
assert(out:write(K.row('verdict',{status=status,reason=T.hex(failed or (selected and (mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect') or (awaiting_birth and 'locked cohort lacks earlier observed eligible query' or 'no eligible normal moving cohort')))})..'\n'));assert(raw:close());assert(out:close())
assert(status=='PASS','normal-fire-session: '..status..': '..tostring(failed or 'fixed window has no eligible cohort'))
