-- TEST-ONLY read-only collector over session:wpn_events(). No boot, launch,
-- provider, policy, actor selection or output operation. Its caller supplies
-- capture/frame context; tick is the existing simulated clock, not a new native
-- frame ordinal. Normal capture has its own sequence, independent of wpn-ev.
local M = {}
local schemas = {
 normal_fire='fire launcher target weapon ready actual intercept angle maximum',
 normal_detach='fire detach launcher weapon config package devices burst ammo_before ammo_after visible_before visible_after flash_present flash_before flash_after volume ground pose',
 normal_birth='fire detach w launcher target weapon matched pose',
 normal_end='fire births', normal_error='reason'
}
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
local function identity(s) if not s:match('^eech%-world:[1-9]%d*$') then error('identity',0) end return s end
local function parse(line)
 local body=line:match('^normal%-ev (.+)$');if not body then error('event prefix',0) end
 local e={};for token in body:gmatch('%S+') do
  local k,v=token:match('^([a-z_]+)=(%S+)$');if not k or e[k] then error('duplicate/malformed field',0) end;e[k]=v
 end
 local spec=schemas[e.kind];if not spec then error('event kind',0) end
 local allowed={seq=true,tick=true,kind=true};for key in spec:gmatch('%S+') do allowed[key]=true;if not e[key] then error('missing '..key,0) end end
 for key in pairs(e) do if not allowed[key] then error('extra '..key,0) end end
 e.seq=integer(e.seq,1,9007199254740991);e.tick=integer(e.tick,0,4294967295)
 return e
end
function M.new(capture,frame_ms)
 assert(type(capture)=='string' and #capture>0 and #capture<=128 and capture:match('^[%w_.:-]+$'),'capture identity')
 assert(type(frame_ms)=='number' and frame_ms>=1 and frame_ms<=4294967295 and frame_ms==math.floor(frame_ms),'frame duration')
 local self={};local problem,closed,sequence,frame,clock,active,last_fire;local completed={};local ordinals={}
 local function fail(reason) problem=problem or ('normal-fire: '..tostring(reason));error(problem,0) end
 local function open() if problem then error(problem,0) end;if closed then fail('continuation after finalization') end end
 function self:feed(current_frame,expected_tick,text)
  open()
  local ok,why=pcall(function()
   if type(current_frame)~='number' or current_frame<1 or current_frame~=math.floor(current_frame) or
      type(expected_tick)~='number' or expected_tick<0 or expected_tick>4294967295 or expected_tick~=math.floor(expected_tick) then error('frame context',0) end
   if frame and (current_frame<frame or (current_frame==frame and expected_tick~=clock)) then error('frame context changed',0) end
   if frame and current_frame>frame and expected_tick~=(clock+(current_frame-frame)*frame_ms)%4294967296 then error('frame clock relation',0) end
   if active and current_frame~=frame then error('open fire crossed frame',0) end
   frame,clock=current_frame,expected_tick
   if type(text)~='string' or (#text>0 and text:sub(-1)~='\n') then error('event framing',0) end
   for line in text:gmatch('[^\n]+') do
    if line:match('^normal%-ev ') then
     local e=parse(line)
     if sequence and e.seq~=sequence+1 then error('capture sequence',0) end;sequence=e.seq
     if e.tick~=clock then error('capture clock',0) end
     if e.kind=='normal_error' then error('observer '..e.reason,0) end
     e.fire=integer(e.fire,0,9007199254740991)
     if e.kind=='normal_fire' then
      if active or e.fire==0 or (last_fire and e.fire<=last_fire) then error('fire context',0) end;last_fire=e.fire
      identity(e.launcher);identity(e.target);if e.launcher==e.target then error('self target',0) end
      e.weapon=integer(e.weapon,1,2147483647);integer(e.ready,1,1)
      words(e.actual,3,true);words(e.intercept,3,true);words(e.angle,1,true);words(e.maximum,1,true)
      e.frame=frame;e.capture=capture;e.birth_list={};active=e
     elseif e.kind=='normal_detach' then
      integer(e.detach,1,9007199254740991);local detach_weapon=integer(e.weapon,1,2147483647);local config=integer(e.config,-1,2147483647);local package=integer(e.package,-1,31)
      local mask=integer(e.devices,0,15);if not ({[0]=true,[1]=true,[3]=true,[7]=true,[11]=true,[15]=true})[mask] then error('device path',0) end
      local burst=integer(e.burst,1,2147483647);local before=integer(e.ammo_before,-1,2147483647);local after=integer(e.ammo_after,-1,2147483647)
      if package>=0 and (config<0 or before<=0 or after~=math.max(0,before-burst)) then error('package effect',0) end
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
      e.w=integer(e.w,1,2147483647);words(e.pose,12,true)
      if e.fire>0 then
       identity(e.launcher);identity(e.target);integer(e.matched,1,1)
       if not active or e.fire~=active.fire or not active.pending or e.detach~=active.pending.detach or
          e.launcher~=active.launcher or e.target~=active.target or integer(e.weapon,1,2147483647)~=active.weapon or ordinals[e.w] then error('birth identity/context',0) end
       ordinals[e.w]=true;e.detachment=active.pending;active.pending=nil;active.birth_list[#active.birth_list+1]=e
      elseif active then error('unrelated birth inside fire',0) end
     elseif e.kind=='normal_end' then
      if not active or e.fire~=active.fire or active.pending or integer(e.births,0,2147483647)~=#active.birth_list then error('end context',0) end
      completed[#completed+1]=active;active=nil
     end
    elseif not line:match('^wpn%-ev ') then error('unexpected event line',0) end
   end
  end)
  if not ok then fail(why) end
 end
 function self:failure(reason) open();fail('native failure: '..tostring(reason)) end
 function self:finish()
  open();if active then fail('incomplete fire') end;closed=true
  local births=0;for _,fire in ipairs(completed) do births=births+#fire.birth_list end
  return {capture=capture,fires=completed,births=births,status=births>0 and 'CORRELATED' or 'UNREACHED'}
 end
 return self
end
return M
