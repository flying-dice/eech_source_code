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
local reference
if mode=='terrain_lift' then
 local f=assert(io.open(assert(a.reference),'rb'));local text=f:read('*a');assert(f:close())
 local expected={};for k,v in pairs(identity) do expected[k]=v end
 expected.mode='healthy';expected.capture=identity.capture:gsub(':terrain_lift$',':healthy');expected.reference='none'
 reference=K.healthy(K.parse(text),expected,T) -- FULL gate before output/provider/boot
 assert(identity.reference~='none' and #identity.reference==64,'reference digest')
else assert(a.reference==nil and identity.reference=='none','healthy reference') end
local out=assert(host.create_exclusive(a.out));local raw=assert(host.create_exclusive(a.out..'.events'))
local capture=N.new(identity.capture,100);local engine,failed,frame= nil,nil,0
local registrations,scenes,keys={},{},{};local bind_count,changed,moving=0,0,0
local records,moves={},{};local selected,evidence,previous,current
local ZERO,ONE,TWO='0x00000000','0x3f800000','0x40000000'
local powers={[0]=1};for i=1,31 do powers[i]=powers[i-1]*2 end
local function whole(n)
 if n==0 then return ZERO end;local e=0;while powers[e+1] and powers[e+1]<=n do e=e+1 end
 return string.format('0x%08x',(127+e)*8388608+(n-powers[e])*powers[23-e])
end
local function mask(lane) local digit=math.floor(lane/4);return '0x'..string.rep('0',7-digit)..string.format('%x',powers[lane%4])..string.rep('0',digit) end
local function issue(why) failed=failed or tostring(why) end
local function finite(x) return type(x)=='number' and x==x and math.abs(x)<math.huge end
local function vector(v) return type(v)=='table' and #v==3 and finite(v[1]) and finite(v[2]) and finite(v[3]) end
local function csv(v) return string.format('%.17g,%.17g,%.17g',v[1],v[2],v[3]) end
local function flag(key,answer)
 local head,id=key:match('^(%S+) ?(.*)$');local w=K.patterns(head,16);if not w then return end
 local phase;for n=19,22 do if w[12]==whole(n) then phase=n end end;if not phase then return end
 local p=assert(K.patterns(answer.p,3),'flag reply');local arr=K.number(w[13]);local lane=K.number(w[14])
 for i=1,16 do if i~=12 and not (phase==19 and i==1) and not (phase~=19 and i==13) and not ((phase==20 or phase==21) and i==14) then assert(w[i]==ZERO,'flag request grammar') end end
 if phase==19 then
  assert(id=='' and (w[1]==ZERO or w[1]==ONE) and answer.result=='flag_group' and p[1]==w[1] and p[2]==ZERO and p[3]==ZERO and type(answer.name)=='string' and answer.name:match('^f[1-9]%d*$') and #answer.name<=20 and not keys[answer.name],'group ACK')
  keys[answer.name]={{},{},{}};return {phase=phase,key=answer.name}
 end
 assert(arr>=0 and arr<=2 and arr==math.floor(arr) and p[1]==w[13],'flag array')
 if phase==22 then
  assert(id=='' and answer.result=='flag_reset' and p[3]==ZERO,'reset ACK');for _,group in pairs(keys) do group[arr+1]={} end;return {phase=phase}
 end
 assert(lane>=0 and lane<=31 and lane==math.floor(lane) and p[2]==w[14] and keys[id],'flag key/lane')
 local lanes=keys[id][arr+1];local expected=phase==20 and mask(lane) or (lanes[lane] and mask(lane) or ZERO)
 assert(answer.result==(phase==20 and 'flag_set' or 'flag_read') and p[3]==expected,'real store transcript')
 if phase==20 then lanes[lane]=true else keys[id][arr+1]={[lane]=lanes[lane]} end
 return {phase=phase,key=id,array=arr,lane=lane,word=p[3]}
end
local function model(op,scene,depth,kind,definition,key)
 local call=capture:model_entry(op)
 local answer=host.model(op,scene,depth,kind,definition,key) -- exactly once, unchanged
 local r={op=op,scene=scene,answer=answer}
 local ok,why=pcall(function()
  if op=='scene' then assert(type(answer.name)=='string','scene name');scenes[scene]=answer.name
  elseif op=='entity_candidate' then
   r.q=assert(T.request(scene,depth,kind,definition,key,K.patterns))
   local err,refused=T.answer(r.q,answer,scenes,K.patterns);assert(not err,err)
   if refused then issue('real candidate refusal: '..answer.reason) end
  elseif op=='entity_scan' and type(key)=='string' then r.flag=flag(key,answer) end
 end)
 if not ok then issue(why) end
 records[call]=r;return answer
end
local function move(step)
 local delegated=step
 if reference and frame==tonumber(reference.frame) and type(step)=='table' and step.host_id==reference.target and step.mode=='move' then
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
 if not b or not f or not f.canonical or f.frame<=300 or not st.same_target or not st.same_launcher or not registrations[b.launcher] then return end
 local m=moves[b.target];local registration=registrations[b.target]
 if not m or not registration or not finite(m.base) or not finite(m.ceiling) or m.base+500>m.ceiling or m.motion[1]^2+m.motion[2]^2+m.motion[3]^2<=1 then return end
 for i,event in ipairs(query.models) do
  local r=records[event.call];assert(r,'missing live provider entry')
  if r.q and r.q.identity==b.target and T.eligible(r.q) and r.answer.result=='need_updated' then
   local j=i+1;local next_r=query.models[j] and records[query.models[j].call]
   if next_r and next_r.flag and next_r.flag.phase==19 then j=j+1;next_r=query.models[j] and records[query.models[j].call] end
   if next_r and next_r.flag and next_r.flag.phase==21 and next_r.flag.array==0 then
    local read=next_r.flag;local carry_event=query.models[j+1];local carry=carry_event and records[carry_event.call]
    if carry and carry.q and not T.same_reask({scene=r.q.scene,identity=b.target,words=r.q.words,answered_word=read.word},carry.q) then
     for k=j+1,#query.models do
      local test_event=query.models[k];local test=records[test_event.call]
      if test.q and test.q.identity==b.target and test.q.scene==r.q.scene and test.answer.result=='test' and K.patterns(test.answer.p,16) then
       local s={frame=frame,birth_frame=f.frame,fire=b.fire,detach=b.detach,w=b.w,step=query.step,query=query.query,before=query.before,after=query.result.after,need=event.call,read=query.models[j].call,carry=carry_event.call,test=test_event.call,launcher=b.launcher,target=b.target,weapon=b.weapon,scene=r.q.scene,name=T.hex(r.answer.name),member=registration.member,group=registration.group,bind=registration.bind,key=read.key,lane=read.lane,word=read.word,fact=carry.q.words[21],pose=test.answer.p,request=table.concat(r.q.words,','),reask=table.concat(carry.q.words,','),position=csv(m.position),motion=csv(m.motion),base=string.format('%.17g',m.base),ceiling=string.format('%.17g',m.ceiling),previous=string.format('%.17g',previous),current=string.format('%.17g',current)}
       local ev={fire=T.hex(f.literal),detach=T.hex(b.detachment.literal),birth=T.hex(b.literal),step=T.hex(st.literal),query=T.hex(query.literal),query_end=T.hex(query.result.literal),step_end=T.hex(st.result.literal)}
       local lines={};for _,v in ipairs(query.models) do lines[#lines+1]=v.literal end;ev.models=T.hex(table.concat(lines,'\n')..'\n')
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
 local clock=engine:clock();assert(clock.elapsed_seconds==0,'fresh seed1 clock')
 local boot=engine:wpn_events();assert(raw:write(boot));capture:feed(1,0,boot);records={};moves={}
 for n=1,1500 do
  frame=n;previous=clock.elapsed_seconds;engine:frame(100);clock=engine:clock();current=clock.elapsed_seconds
  assert(finite(previous) and finite(current) and current>previous,'real frame clock')
  local events=engine:wpn_events();assert(raw:write(events))
  local queries=capture:feed(frame,(frame-1)*100,events)
  for _,q in ipairs(queries) do choose(q) end
  records={};moves={}
 end
 if not failed then engine:close();engine=nil end -- no getter/close after failure
end,function(e)return tostring(e)end)
if not ok then issue(why) end
local finished;local clean,err=pcall(function()finished=capture:finish()end);if not clean then issue(err) end
local counts={frames=frame,fires=finished and #finished.fires or 0,births=finished and finished.births or 0,steps=finished and #finished.steps or 0,queries=finished and #finished.queries or 0,model_calls=finished and finished.model_calls or 0,moving=moving,failed=failed and 1 or 0,changed=changed}
local status=failed and 'FAIL' or (selected and 'PASS' or 'UNREACHED')
if reference and status=='PASS' then
 for _,k in ipairs({'frame','birth_frame','fire','detach','w','step','query','launcher','target','weapon','scene','member','group','bind'}) do if tostring(selected[k])~=reference[k] then issue('changed selected identity '..k) end end
 if changed~=1 or selected.pose==reference.pose or selected.position==reference.position then issue('terrain input has no same-cohort pose effect') end
 if failed then counts.failed=1;status='FAIL' end
end
if status=='PASS' then
 local text=K.row('identity',identity)..'\n'..K.row('counts',counts)..'\n'..K.row('selection',selected)..'\n'..K.row('evidence',evidence)..'\n'..K.row('verdict',{status=status,reason=T.hex(mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect')})..'\n'
 local valid,problem=pcall(function()return K.validate(K.parse(text),{},T)end)
 if not valid then issue(problem);counts.failed=1;status='FAIL' end
end
assert(out:write(K.row('identity',identity)..'\n'..K.row('counts',counts)..'\n'))
if selected then assert(out:write(K.row('selection',selected)..'\n'..K.row('evidence',evidence)..'\n')) end
assert(out:write(K.row('verdict',{status=status,reason=T.hex(failed or (selected and (mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect') or 'no eligible normal moving cohort'))})..'\n'));assert(raw:close());assert(out:close())
assert(status=='PASS','normal-fire-session: '..status..': '..tostring(failed or 'fixed window has no eligible cohort'))
