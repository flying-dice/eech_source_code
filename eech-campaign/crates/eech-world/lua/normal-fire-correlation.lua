-- TEST-ONLY read-only collector over session:wpn_events(). No boot, launch,
-- provider, policy, actor selection or output operation. Its caller supplies
-- capture/frame context; tick is the existing simulated clock, not a new native
-- frame ordinal. Normal capture has its own sequence, independent of wpn-ev.
local M = {}
local schemas = {
 normal_fire='fire launcher target weapon ready actual intercept angle maximum',
 normal_detach='fire detach launcher weapon config package devices burst ammo_before ammo_after visible_before visible_after flash_present flash_before flash_after volume ground pose',
 normal_birth='fire detach w launcher target weapon matched pose',
 normal_end='fire births', normal_error='reason',
 normal_step='step w launcher target weapon pose motion velocity lifetime code',
 normal_step_end='step w launcher target weapon pose motion velocity lifetime code',
 normal_query='step query w which before segment', normal_query_end='step query w after hit hit_id',
 normal_model='step query w call op'
}
local model_ops={catalogue=1,scene=2,triangle=3,route_node=4,route_face=5,object_table=6,object=7,animation=8,custom_object=9,custom_order=10,
 target_point_offset=11,target_point_chain=12,target_point_world=13,target_point_fixed=14,entity_collision=15,entity_candidate=16,entity_scan=17,entity_pose=18}
local function integer(s,lo,hi)
 if type(s)~='string' or not s:match('^%-?%d+$') then error('integer',0) end
 local v=tonumber(s); if not v or v<lo or v>hi or v~=math.floor(v) or tostring(v)~=s then error('integer domain',0) end
 return v
end
local function words(s,n,finite)
 local out={}; for w in (s..','):gmatch('(.-),') do
  if #w~=10 or not w:match('^0x[0-9a-f]+$') then error('raw word',0) end
  local v=tonumber(w:sub(3),16)
  if finite and math.floor(v/8388608)%256==255 then error('nonfinite physical word',0) end
  out[#out+1]=w
 end
 if #out~=n then error('word count',0) end
 return out
end
local function identity(s) if #s>128 or not s:match('^eech%-world:[1-9]%d*$') then error('identity',0) end return s end
local function parse(line)
 local body=line:match('^normal%-ev (.+)$');if not body then error('event prefix',0) end
 local e={};for token in body:gmatch('%S+') do
  local k,v=token:match('^([a-z_]+)=(%S+)$');if not k or e[k] then error('duplicate/malformed field',0) end;e[k]=v
 end
 local spec=schemas[e.kind];if not spec then error('event kind',0) end
 local allowed={seq=true,tick=true,kind=true};for key in spec:gmatch('%S+') do allowed[key]=true;if not e[key] then error('missing '..key,0) end end
 for key in pairs(e) do if not allowed[key] then error('extra '..key,0) end end
 local canonical='normal-ev seq='..e.seq..' tick='..e.tick..' kind='..e.kind
 for key in spec:gmatch('%S+') do canonical=canonical..' '..key..'='..e[key] end
 assert(canonical==line,'event field order')
 e.seq=integer(e.seq,1,9007199254740991);e.tick=integer(e.tick,0,4294967295)
 return e
end
-- Semantic event domains, shared by live capture and selected report excerpts.
-- Bounds are the unchanged compiled enums in en_sbtyp.h and weapon.h at2fc.
function M.event(line)
 local e=parse(line);local function I(k,lo,hi)return integer(e[k],lo,hi or 2147483647)end
 for _,k in ipairs({'launcher','target','hit_id'}) do if e[k] and e[k]~='none' then identity(e[k]) end end
 if e.fire then I('fire',0,9007199254740991) end
 if e.weapon then I('weapon',1,82) end
 if e.w then I('w',1) end
 if e.step then I('step',1,9007199254740991) end
 if e.query then I('query',1,9007199254740991) end
 if e.kind=='normal_fire' then
  I('fire',1,9007199254740991);I('ready',1,1)
  words(e.actual,3,true);words(e.intercept,3,true);words(e.angle,1,true);words(e.maximum,1,true)
 elseif e.kind=='normal_detach' then
  I('detach',1,9007199254740991);local config=I('config',-1,740);local package=I('package',-1,31)
  local mask=I('devices',0,15);assert(({[0]=true,[1]=true,[3]=true,[7]=true,[11]=true,[15]=true})[mask],'device path')
  local burst=I('burst',1);local before=I('ammo_before',-1);local after=I('ammo_after',-1)
  if package>=0 then assert(config>=0 and before>0 and after==math.max(0,before-burst),'package effect')
  else assert(config==-1 and before==-1 and after==-1 and mask==0,'absent package effect') end
  local vb=I('visible_before',-1,1);local va=I('visible_after',-1,1)
  if math.floor(mask/4)%2==1 then assert(vb>=0 and va==0,'station effect') else assert(vb==-1 and va==-1,'absent station effect') end
  local flash=I('flash_present',0,1);words(e.flash_before,1,true);words(e.flash_after,1,true)
  if flash==0 then assert(e.flash_before=='0x00000000' and e.flash_after=='0x00000000','absent flash effect')
  else
   assert(mask>=8,'flash without muzzle')
   if tonumber(e.flash_before:sub(3),16)%2147483648==0 then assert(tonumber(e.flash_after:sub(3),16)>=2147483648,'new flash sign')
   else assert(e.flash_after==e.flash_before,'ongoing flash changed') end
  end
  local volume=I('volume',0,1);I('ground',0,1);assert(volume==0 or mask>=3,'volume fallback without device');words(e.pose,12,true)
 elseif e.kind=='normal_birth' then
  I('detach',1,9007199254740991);local matched=I('matched',0,1);assert((e.fire=='0')==(matched==0),'birth match domain');words(e.pose,12,true)
 elseif e.kind=='normal_end' then I('fire',1,9007199254740991);I('births',0)
 elseif e.kind=='normal_step' or e.kind=='normal_step_end' then
  words(e.pose,12,true);words(e.motion,3,true);words(e.velocity,1,true);words(e.lifetime,1,true);I('code',0,7)
 elseif e.kind=='normal_query' then I('which',0);I('before',0,9007199254740991);words(e.segment,6,true)
 elseif e.kind=='normal_model' then I('call',1,9007199254740991);I('op',1,18)
 elseif e.kind=='normal_query_end' then
  I('after',0,9007199254740991);local hit=I('hit',0,1);assert(hit==1 or e.hit_id=='none','miss carries hit identity')
 elseif e.kind=='normal_error' then assert(e.reason:match('^[a-z_]+$'),'observer reason') end
 return e
end
function M.new(capture,frame_ms)
 assert(type(capture)=='string' and #capture>0 and #capture<=128 and capture:match('^[%w_.:-]+$'),'capture identity')
 assert(frame_ms==100,'supported frame duration is 100ms')
 local self={};local problem,closed,sequence,frame,clock,active,last_fire;local completed={};local ordinals={}
 local step,query;local steps,queries,fire_records={},{},{};local last_step,last_query,last_model=0,0,0
 local model_calls,model_pending,model_unread,model_cutoff=0,{},0,0
 local function fail(reason) problem=problem or ('normal-fire: '..tostring(reason));error(problem,0) end
 local function open() if problem then error(problem,0) end;if closed then fail('continuation after finalization') end end
 -- Already delivered, completed normal birth; no C getter, lazy registration
 -- or current-frame drain is performed from an intervention callback.
 function self:prefix(w)
  open();if type(w)~='number' or w<1 or w>2147483647 or w~=math.floor(w) then fail('prefix ordinal') end
  local b=ordinals[w];local f=b and fire_records[b.fire]
  if not f or not b.detachment then return nil end
  return {frame=f.frame,fire=f.literal,detach=b.detachment.literal,birth=b.literal}
 end
 -- Called at EVERY entry of the one real Lua model provider, from before boot.
 -- Return the exact ordinal for caller-owned request/reply records. No physical
 -- answer is chosen or changed; a later C record proves which query called it.
 function self:model_entry(op)
  open();local n=model_ops[op];if not n then fail('model operation') end
  model_calls=model_calls+1;model_unread=model_unread+1;model_pending[model_calls]=n
  if model_unread>100000 or model_calls>9007199254740991 then fail('undrained model entries') end
  return model_calls
 end
 function self:feed(current_frame,expected_tick,text)
  open()
  local first_new=#queries+1
  local ok,why=pcall(function()
   if type(current_frame)~='number' or current_frame<1 or current_frame>4294967295 or current_frame~=math.floor(current_frame) or
      type(expected_tick)~='number' or expected_tick<0 or expected_tick>4294967295 or expected_tick~=math.floor(expected_tick) then error('frame context',0) end
   if frame and (current_frame<frame or (current_frame==frame and expected_tick~=clock)) then error('frame context changed',0) end
   if frame and current_frame>frame and expected_tick~=(clock+(current_frame-frame)*frame_ms)%4294967296 then error('frame clock relation',0) end
   if (active or step or query) and current_frame~=frame then error('open interval crossed frame',0) end
   if frame and current_frame>frame then
    for call in pairs(model_pending) do if call<=model_cutoff then model_pending[call]=nil end end
    model_unread=model_calls-model_cutoff
   end
   frame,clock=current_frame,expected_tick
   if type(text)~='string' or (#text>0 and text:sub(-1)~='\n') then error('event framing',0) end
   for line in text:gmatch('[^\n]+') do
    if line:match('^normal%-ev ') then
     local e=M.event(line)
     e.literal=line
     if sequence and e.seq~=sequence+1 then error('capture sequence',0) end;sequence=e.seq
     if e.tick~=clock then error('capture clock',0) end
     if e.kind=='normal_error' then error('observer '..e.reason,0) end
     if e.fire then e.fire=integer(e.fire,0,9007199254740991) end
     if e.kind=='normal_fire' then
      if active or e.fire==0 or (last_fire and e.fire<=last_fire) then error('fire context',0) end;last_fire=e.fire
      if e.launcher~='none' then identity(e.launcher) end;if e.target~='none' then identity(e.target) end
      e.canonical=e.launcher~='none' and e.target~='none'
      if e.canonical and e.launcher==e.target then error('self target',0) end
      e.weapon=integer(e.weapon,1,2147483647);integer(e.ready,1,1)
      words(e.actual,3,true);words(e.intercept,3,true);words(e.angle,1,true);words(e.maximum,1,true)
      e.frame=frame;e.capture=capture;e.birth_list={};active=e
     elseif e.kind=='normal_detach' then
      integer(e.detach,1,9007199254740991);if e.launcher~='none' then identity(e.launcher) end;local detach_weapon=integer(e.weapon,1,2147483647);local config=integer(e.config,-1,2147483647);local package=integer(e.package,-1,31)
      local mask=integer(e.devices,0,15);if not ({[0]=true,[1]=true,[3]=true,[7]=true,[11]=true,[15]=true})[mask] then error('device path',0) end
      local burst=integer(e.burst,1,2147483647);local before=integer(e.ammo_before,-1,2147483647);local after=integer(e.ammo_after,-1,2147483647)
      if package>=0 and (config<0 or before<=0 or after~=math.max(0,before-burst)) then error('package effect',0) end
      if package==-1 and (config~=-1 or before~=-1 or after~=-1 or mask~=0) then error('absent package effect',0) end
      local vb=integer(e.visible_before,-1,1);local va=integer(e.visible_after,-1,1)
      if math.floor(mask/4)%2==1 then if vb<0 or va~=0 then error('station effect',0) end elseif vb~=-1 or va~=-1 then error('absent station effect',0) end
      local flash=integer(e.flash_present,0,1);words(e.flash_before,1,false);words(e.flash_after,1,false)
      if flash==1 and mask<8 then error('flash without muzzle',0) end
      integer(e.volume,0,1);integer(e.ground,0,1);words(e.pose,12,true)
      if e.fire>0 then
       if not active or e.fire~=active.fire or e.launcher~=active.launcher or detach_weapon~=active.weapon or active.pending or package<0 then error('detach context',0) end
       active.pending=e
      elseif active then error('unrelated detach inside fire',0) end
     elseif e.kind=='normal_birth' then
      e.w=integer(e.w,1,2147483647);integer(e.detach,1,9007199254740991);integer(e.weapon,1,2147483647);local matched=integer(e.matched,0,1);words(e.pose,12,true)
      if e.launcher~='none' then identity(e.launcher) end;if e.target~='none' then identity(e.target) end
      if e.fire==0 and matched~=0 then error('unmatched birth domain',0) end
      if e.fire>0 then
       if e.launcher~='none' then identity(e.launcher) end;if e.target~='none' then identity(e.target) end;integer(e.matched,1,1)
       if not active or e.fire~=active.fire or not active.pending or e.detach~=active.pending.detach or
          e.launcher~=active.launcher or e.target~=active.target or integer(e.weapon,1,2147483647)~=active.weapon or ordinals[e.w] then error('birth identity/context',0) end
       ordinals[e.w]=e;e.detachment=active.pending;active.pending=nil;active.birth_list[#active.birth_list+1]=e
      elseif active then error('unrelated birth inside fire',0) end
     elseif e.kind=='normal_end' then
      if not active or e.fire~=active.fire or active.pending or integer(e.births,0,2147483647)~=#active.birth_list then error('end context',0) end
      completed[#completed+1]=active;fire_records[active.fire]=active;active=nil
     elseif e.kind=='normal_step' or e.kind=='normal_step_end' then
      e.step=integer(e.step,1,9007199254740991);e.w=integer(e.w,1,2147483647);integer(e.weapon,1,2147483647)
      if e.launcher~='none' then identity(e.launcher) end;if e.target~='none' then identity(e.target) end
      words(e.pose,12,true);words(e.motion,3,true);words(e.velocity,1,true);words(e.lifetime,1,true);integer(e.code,0,2147483647)
      if e.kind=='normal_step' then
       if active or step or e.step<=last_step then error('step context',0) end
       last_step=e.step;e.frame=frame;e.capture=capture;e.birth=ordinals[e.w];e.queries={};step=e
       if e.birth and e.weapon~=e.birth.weapon then error('step weapon identity',0) end
       -- Loss/retargeting is real campaign behavior, not malformed capture.
       -- A later SAME-target witness must require both of these facts.
       e.same_target=e.birth and e.target==e.birth.target or false
       e.same_launcher=e.birth and e.launcher==e.birth.launcher or false
      else
       if not step or query or e.step~=step.step or e.w~=step.w or e.launcher~=step.launcher or e.weapon~=step.weapon then error('step end context',0) end
       step.result=e;steps[#steps+1]=step;step=nil
      end
     elseif e.kind=='normal_query' then
      e.step=integer(e.step,1,9007199254740991);e.query=integer(e.query,1,9007199254740991);e.w=integer(e.w,1,2147483647);integer(e.which,0,2147483647);words(e.segment,6,true)
      e.before=integer(e.before,0,model_calls)
      if not step or query or e.step~=step.step or e.w~=step.w or e.query<=last_query then error('query context',0) end
      last_query=e.query;e.models={};e.birth=step.birth;e.step_record=step;e.normal_fire=e.birth and fire_records[e.birth.fire] or nil;e.frame=frame;e.capture=capture;query=e
     elseif e.kind=='normal_model' then
      e.step=integer(e.step,1,9007199254740991);e.query=integer(e.query,1,9007199254740991);e.w=integer(e.w,1,2147483647)
      e.call=integer(e.call,1,9007199254740991);e.op=integer(e.op,1,18)
      if not query or e.step~=step.step or e.query~=query.query or e.w~=step.w or e.call<=last_model or e.call~=query.before+#query.models+1 or model_pending[e.call]~=e.op then error('model callback join',0) end
      last_model=e.call;query.models[#query.models+1]=e
     elseif e.kind=='normal_query_end' then
      e.step=integer(e.step,1,9007199254740991);e.query=integer(e.query,1,9007199254740991);e.w=integer(e.w,1,2147483647);integer(e.hit,0,1)
      if e.hit_id~='none' then identity(e.hit_id);if e.hit~='1' then error('hit identity',0) end end
      e.after=integer(e.after,0,model_calls)
      if not query or e.step~=step.step or e.query~=query.query or e.w~=step.w then error('query end context',0) end
      if e.after~=query.before+#query.models then error('incomplete model interval',0) end
      query.result=e;step.queries[#step.queries+1]=query;queries[#queries+1]=query;query=nil
     end
    elseif not line:match('^wpn%-ev ') then error('unexpected event line',0)
    elseif line:match(' kind=truncated') then error('legacy event truncation',0) end
   end
   model_cutoff=model_calls
  end)
  if not ok then fail(why) end
  local added={};for i=first_new,#queries do added[#added+1]=queries[i] end;return added
 end
 function self:failure(reason) open();fail('native failure: '..tostring(reason)) end
 function self:finish()
  open();if active or step or query then fail('incomplete normal path') end;closed=true
  local births=0;for _,fire in ipairs(completed) do births=births+#fire.birth_list end
  return {capture=capture,fires=completed,births=births,steps=steps,queries=queries,model_calls=model_calls,status=births>0 and 'CORRELATED' or 'UNREACHED'}
 end
 return self
end
return M
