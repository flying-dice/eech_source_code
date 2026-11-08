local K=assert(loadfile(base..'/normal-fire-session-contract.lua'))()
local N=assert(loadfile(base..'/normal-fire-correlation.lua'))();K.event=N.event
local T=assert(loadfile(base..'/physical-flags-read-transaction.lua'))()
local rf=assert(io.open(retained,'rb'));local real_events=rf:read('*a');rf:close()
local codec=N.new('retained-real-C',100);codec:feed(1,0,real_events);assert(codec:prefix(1))
local original=codec:finish();assert(original.births==1 and #original.queries==1) -- replay codec only, no new C execution
-- Real reference-provider traffic with the same entity_scan op as the retained
-- native boot failure; the empty-store RESET is a valid metadata request.
-- This is boot-shaped private traffic, not a retail/campaign boot claim.
local reset_words={};for i=1,16 do reset_words[i]='0x00000000' end;reset_words[12]=K.whole(22)
local reset_key=table.concat(reset_words,',')
local function boot_traffic(c,count,offset)
 for i=1,count do
  assert(c:model_entry('entity_scan')==(offset or 0)+i)
  local a=host.model('entity_scan',0,nil,nil,nil,reset_key)
  assert(a.result=='flag_reset' and a.p=='0x00000000,0x00000000,0x00000000')
 end
end
local small=N.new('short-real-provider-boot',100);boot_traffic(small,32);small:feed(1,0,'');small:drained();assert(small:finish().model_calls==32)
local long=N.new('long-real-provider-boot',100);boot_traffic(long,100001);long:feed(1,0,'');long:drained()
boot_traffic(long,100001,100001) -- first frame calls happen BEFORE its feed, at the SAME frame/tick as the boot drain
local function query_stream(before,op,step,tick)
 local st=assert(real_events:match('(normal%-ev [^\n]+kind=normal_step step=2 [^\n]+)'))
 local qe=assert(real_events:match('(normal%-ev [^\n]+kind=normal_query_end [^\n]+)'))
 local en=assert(real_events:match('(normal%-ev [^\n]+kind=normal_step_end step=2 [^\n]+)'))
 local qu=assert(real_events:match('(normal%-ev [^\n]+kind=normal_query [^\n]+)'))
 -- Synthetic sequence/context over retained actual physical fields, not a new C query.
 local rows={st,qu,'normal-ev seq=3 tick=0 kind=normal_model step=2 query=1 w=1 call='..(before+1)..' op='..op,qe,en}
 for i,line in ipairs(rows) do rows[i]=line:gsub('seq=%d+','seq='..((step-1)*5+i),1):gsub('tick=0','tick='..tick):gsub('step=2','step='..step):gsub('query=1','query='..step):gsub('before=0','before='..before):gsub('after=0','after='..(before+1)) end
 return table.concat(rows,'\n')..'\n'
end
boot_traffic(long,1,200002);local first=query_stream(200002,17,1,0)
assert(#long:feed(1,0,first)==1);long:drained()
boot_traffic(long,1,200003);assert(#long:feed(2,100,query_stream(200003,17,2,100))==1);long:drained()
assert(long:finish().model_calls==200004)
local split=N.new('partial-drain',100);split:model_entry('entity_scan')
local stream=query_stream(0,17,1,0);local cut=assert(stream:find('normal%-ev seq=3'))
split:feed(1,0,stream:sub(1,cut-1));split:feed(1,0,stream:sub(cut));split:drained();assert(split:finish().model_calls==1)
local function latch(c,operation,needle)
 local ok,why=pcall(operation);assert(not ok and tostring(why):find(needle,1,true),tostring(why))
 for _,resume in ipairs({function()c:drained()end,function()c:prefix(1)end,function()c:feed(1,0,'')end,function()c:finish()end,function()c:model_entry('scene')end,function()c:failure('other')end}) do
  local accepted,same=pcall(resume);assert(not accepted and same==why)
 end
end
local partial=N.new('partial-ACK',100);partial:model_entry('entity_scan');partial:feed(1,0,stream:sub(1,cut-1));latch(partial,function()partial:drained()end,'incomplete model drain')
local late=N.new('late-ACK',100);late:feed(1,0,'');late:model_entry('entity_scan');latch(late,function()late:drained()end,'incomplete model drain')
local wrong=N.new('wrong-compressed-op',100);wrong:model_entry('entity_scan');latch(wrong,function()wrong:feed(1,0,query_stream(0,18,1,0))end,'model callback join')
local alternation=N.new('alternating-budget',100);for i=1,100000 do alternation:model_entry(i%2==0 and 'entity_scan' or 'entity_pose') end
latch(alternation,function()alternation:model_entry('entity_pose')end,'undrained model operation runs')
local retired=N.new('retired-budget',100);for i=1,100000 do retired:model_entry(i%2==0 and 'entity_scan' or 'entity_pose') end
retired:feed(1,0,'');retired:drained();assert(retired:model_entry('entity_pose')==100001);retired:feed(2,100,'');retired:drained();assert(retired:finish().model_calls==100001)
local poisoned=N.new('prefix-latch',100);assert(poisoned:prefix(1)==nil)
local ok,reason=pcall(function()poisoned:model_entry('bad_operation')end);assert(not ok)
for _,resume in ipairs({function()poisoned:drained()end,function()poisoned:prefix(1)end,function()poisoned:feed(1,0,'')end,function()poisoned:finish()end,function()poisoned:model_entry('scene')end,function()poisoned:failure('other')end}) do
 local accepted,same=pcall(resume);assert(not accepted and same==reason)
end
local f=assert(io.open(positive,'rb'));local text=f:read('*a');f:close()
assert(K.healthy(K.parse(text),{},T).target=='eech-world:2') -- coherent codec positive FIRST
local faults={
 {'detach','devices','garbage'},{'detach','devices','2'},{'detach','ammo_after','garbage'},{'detach','ammo_after','2'},
 {'detach','config','741'},{'detach','package','32'},{'detach','burst','0'},{'detach','visible_after','1'},
 {'detach','flash_present','0'},{'detach','flash_before','0x3f800000'},{'detach','ground','2'},
 {'fire','actual','0x7fc00000,0x00000000,0x3f800000'},{'fire','ready','0'},{'birth','matched','0'},
 {'step','code','8'},{'step','velocity','0x7f800000'},{'step_end','target','eech-world:3'},
 {'step_end','launcher','eech-world:3'},{'step_end','weapon','71'},{'query','which','-1'},
 {'query_end','hit_id','eech-world:2'},{'models','seq','6'},{'models','seq','9999'},{'models','seq','garbage'},{'models','op','19'}}
local function damage(triple)
 local r=K.parse(text);local event=K.unhex(r.evidence[triple[1]])
 local changed,n=event:gsub(triple[2]..'=[^%s]+',triple[2]..'='..triple[3],1);assert(n==1)
 r.evidence[triple[1]]=T.hex(changed);local rows={}
 for _,kind in ipairs({'identity','counts','selection','evidence','verdict'}) do rows[#rows+1]=K.row(kind,r[kind]) end
 return table.concat(rows,'\n')..'\n'
end
for _,triple in ipairs(faults) do assert(not pcall(function()K.healthy(K.parse(damage(triple)),{},T)end)) end
for _,pair in ipairs({{'target=eech%-world:2','target=eech-world:3'},{'carry=13','carry=14'},{'failed=0','failed=1'},{'frames=1500','frames=1499'},{'position=10,100,20','position=11,100,20'}}) do
 local bad,n=text:gsub(pair[1],pair[2],1);assert(n==1 and not pcall(function()K.healthy(K.parse(bad),{},T)end))
end
local calls={prepare=0,boot=0,frame=0,close=0,output=0};local fatal=false
package.preload.eech_dc=function()return {
 prepare_installation=function()calls.prepare=calls.prepare+1 end,
 boot=function(options)
  calls.boot=calls.boot+1;local count=0
  assert(type(options.aim)=='function' and type(options.model)=='function' and type(options.move_vehicle)=='function')
  return {frame=function(_,ms)assert(ms==100);count=count+1;calls.frame=count;if fatal and count==10 then error('unrelated native stand-in failure')end end,
   clock=function()return {elapsed_seconds=count/10}end,
   wpn_events=function()return ''end,
   close=function()calls.close=calls.close+1 end}
 end}end
local create=host.create_exclusive;host.create_exclusive=function()calls.output=calls.output+1;error('output touched')end
local function args(mode,out,reference)
 return {root=root,out=root..out,mode=mode,capture='fixture:'..mode,runtime=string.rep('b',40),bundle=string.rep('a',64),driver_sha256=string.rep('a',64),collector=base..'/normal-fire-correlation.lua',collector_sha256=string.rep('a',64),transaction=base..'/physical-flags-read-transaction.lua',transaction_sha256=string.rep('a',64),contract=base..'/normal-fire-session-contract.lua',contract_sha256=string.rep('a',64),reference=reference,reference_sha256=reference and string.rep('c',64) or nil}
end
host.args=args('terrain_lift','/positive-gate.txt',positive)
local ok,why=pcall(assert(loadfile(base..'/normal-fire-session.lua')))
assert(not ok and tostring(why):find('output touched',1,true) and calls.output==1 and calls.prepare==0)
calls.output=0
local references={damaged,same_frame}
for i,triple in ipairs(faults) do
 local path=root..'/semantic-'..i..'.txt';local file=assert(io.open(path,'wb'));assert(file:write(damage(triple)));assert(file:close());references[#references+1]=path
end
for _,reference in ipairs(references) do
 host.args=args('terrain_lift','/forbidden.txt',reference)
 local accepted,reason=pcall(assert(loadfile(base..'/normal-fire-session.lua')))
 assert(not accepted and not tostring(reason):find('output touched',1,true))
 assert(calls.output==0 and calls.prepare==0 and calls.boot==0 and calls.frame==0)
end
host.create_exclusive=create
-- The actual reference host's store, not a table of copied expected markers.
local Z='0x00000000';local O='0x3f800000';local real_model=host.model;local oracle=K.oracle();local history={}
local function request(phase,array,lane,id,eligible)
 local w={};for i=1,16 do w[i]=Z end;w[12]=K.whole(phase)
 if phase==19 then w[1]=eligible and O or Z else w[13]=K.whole(array);if phase~=22 then w[14]=K.whole(lane) end end
 return table.concat(w,',')..(id and ' '..id or '')
end
local function physical(phase,array,lane,id,eligible)
 local key=request(phase,array,lane,id,eligible);local answer=real_model('entity_scan',0,nil,nil,nil,key)
 oracle:check(key,answer);history[#history+1]={key=key,answer=answer};return answer
end
local low=physical(19,nil,nil,nil,true).name;local high=physical(19,nil,nil,nil,false).name
physical(20,0,1,low);physical(20,0,31,high);physical(20,1,7,low);physical(20,2,8,high)
local reset=physical(22,0);assert(reset.p==Z..','..O..','..Z)
assert(physical(21,0,1,low).p==Z..','..O..','..Z)
assert(physical(21,0,31,high).p==Z..','..K.whole(31)..','..K.mask(31))
assert(physical(21,1,7,low).p==O..','..K.whole(7)..','..K.mask(7))
assert(physical(21,2,8,high).p==K.whole(2)..','..K.whole(8)..','..K.mask(8))
physical(20,0,2,high);physical(20,0,3,high);physical(21,0,2,high)
assert(physical(21,0,3,high).p==Z..','..K.whole(3)..','..Z) -- destructive neighbor erasure
physical(22,0) -- counts the eligible group even when its word is already zero
local function bad_ack(index,p)
 local check=K.oracle();for i=1,index-1 do check:check(history[i].key,history[i].answer) end
 local a={};for k,v in pairs(history[index].answer) do a[k]=v end;a.p=p
 assert(not pcall(function()check:check(history[index].key,a)end))
end
bad_ack(1,Z..','..Z..','..Z);bad_ack(7,Z..','..Z..','..Z);bad_ack(7,O..','..O..','..Z)
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
local baseline=K.parse(text);local unregistered=false;local same_live=false;local other_live=false;local no_later=false;local prefix_fault;local reset_fault
local boot_extra=0;local boot_query=false
local options,binds,candidate_calls,y,lift_calls,move_calls
local function patterns(row)return table.concat(row,',')end
local function flag_request(phase,lane)
 local w={};for i=1,16 do w[i]=Z end
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
 move_calls=move_calls+1
 assert(step.host_id=='eech-world:2' and step.mode=='move');y=step.terrain.elevation
 if y==600 then lift_calls=lift_calls+1 else assert(y==100) end
 return {position={10,y,20},motion_vector={2,0,0}}
end
host.model=function(op,scene,depth,kind,definition,key)
 if op=='scene' then return {name='fixture-scene1'} end
 if op=='entity_scan' then
  if key:find(K.whole(22),1,true) then return {result='flag_reset',p=(reset_fault=='array' and O or Z)..','..(reset_fault=='count' and Z or O)..','..Z} end
  if candidate_calls==0 then return {result='flag_group',name='f1',p=(reset_fault=='eligibility' and Z or O)..','..Z..','..Z} end
  return {result='flag_read',p=Z..','..O..','..Z}
 end
 if op=='entity_candidate' then
  candidate_calls=candidate_calls+1
  if candidate_calls<3 then return {result=candidate_calls==1 and 'need_updated' or 'need_placement',name='fixture-scene1'} end
  local p=K.patterns(baseline.selection.pose,16);if y==600 then p[2]='0x44160000';p[5]='0x44160000' end
  return {result='test',name='fixture-scene1',p=patterns(p)}
 end
 return {result='recorded'}
end
package.loaded.eech_dc=nil
package.preload.eech_dc=function()return {prepare_installation=function()end,boot=function(o)
 options=o;binds=0;candidate_calls=0;y=100;lift_calls=0;move_calls=0
 o.bind_mobile({member_key=1,group_key=1});o.bind_mobile({member_key=2,group_key=1})
 o.model('scene',1);o.model('entity_scan',0,nil,nil,nil,flag_request(19))
 for _=1,8 do o.model('scene',1) end
 for _=1,boot_extra do o.model('scene',1) end -- synthetic Engine boot transport; provider delegates once per call
 if reset_fault then o.model('entity_scan',0,nil,nil,nil,request(22,0)) end
 local count=0;local pending=''
 if boot_query then o.model('entity_pose',1);pending=query_stream(10+boot_extra,18,1,0) end
 local function prefix(tick)
  local e=baseline.evidence;local birth=K.unhex(e.birth);local fire=K.unhex(e.fire)
  if prefix_fault=='birth_pose' then birth=birth:gsub('pose=0x%x+','pose=0x3f800000',1)
  elseif prefix_fault=='fire_actual' then fire=fire:gsub('actual=0x%x+','actual=0x3f800000',1)
  elseif prefix_fault=='wrong_w' then birth=birth:gsub(' w=1 ',' w=2 ',1) end
  return (fire..'\n'..K.unhex(e.detach)..'\n'..birth..'\nnormal-ev seq=4 tick=30000 kind=normal_end fire=1 births=1\n'):gsub('tick=30000','tick='..tick)
 end
 local function query(later)
  local e=baseline.evidence;local tail=K.unhex(e.step)..'\n'..K.unhex(e.query)..'\n'..K.unhex(e.models)..K.unhex(e.query_end)..'\n'..K.unhex(e.step_end)..'\n'
  if later then tail=tail:gsub('seq=(%d+)',function(n)return 'seq='..(tonumber(n)+9)end):gsub('step=1','step=2'):gsub('query=1','query=2'):gsub('before=10','before=15'):gsub('after=15','after=20'):gsub('call=(%d+)',function(n)return 'call='..(tonumber(n)+5)end):gsub('tick=30100','tick=30200') end
  if boot_extra>0 then tail=tail:gsub('before=(%d+)',function(n)return 'before='..(tonumber(n)+boot_extra)end):gsub('after=(%d+)',function(n)return 'after='..(tonumber(n)+boot_extra)end):gsub('call=(%d+)',function(n)return 'call='..(tonumber(n)+boot_extra)end) end
  return tail
 end
 return {clock=function()return {elapsed_seconds=count/10}end,close=function()end,
 frame=function(_,ms)
  assert(ms==100);count=count+1
  if count==301 and not same_live then pending=prefix(30000) end
  if count==302 or (same_live and not no_later and count==303) then
   candidate_calls=0
   local step={host_id='eech-world:2',mode='move',terrain={elevation=100},map_max={4095,10000,4095}}
   o.move_vehicle(step);assert(step.terrain.elevation==100,'caller input mutated')
   o.model('entity_candidate',1,nil,nil,nil,candidate_key(false,false))
   o.model('entity_scan',0,nil,nil,nil,flag_request(21))
   o.model('entity_candidate',1,nil,nil,nil,candidate_key(true,false))
   o.model('entity_pose',1)
   o.model('entity_candidate',1,nil,nil,nil,candidate_key(true,true))
   pending=(same_live and count==302 and prefix(30100) or '')..query(count==303)
   if other_live and count==302 then
    local second=prefix(30100):gsub('seq=(%d+)',function(n)return 'seq='..(tonumber(n)+13)end):gsub('fire=1','fire=2'):gsub('detach=1','detach=2'):gsub(' w=1 ',' w=2 ')
    pending=pending..second
   elseif other_live and count==303 then
    pending=pending:gsub('seq=(%d+)',function(n)return 'seq='..(tonumber(n)+4)end):gsub(' w=1 ',' w=2 ')
   end
   if unregistered then pending=pending:gsub('eech%-world:[12]','none') end
  end
 end,wpn_events=function()local v=pending;pending='';return v end}
 end}end
host.args=args('healthy','/transport-positive.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/transport-positive.txt','rb'));local healthy=e:read('*a');e:close()
assert(K.healthy(K.parse(healthy),{},T) and lift_calls==0)
host.args=args('terrain_lift','/transport-B.txt',root..'/transport-positive.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/transport-B.txt','rb'));local b=K.parse(e:read('*a'));e:close()
assert(K.validate(b,{},T) and b.counts.changed=='1' and lift_calls==1)
assert(K.paired(b,K.parse(healthy)))
boot_extra=100001;host.args=args('healthy','/long-boot-driver.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/long-boot-driver.txt','rb'));local long_driver=K.parse(e:read('*a'));e:close()
assert(K.healthy(long_driver,{},T) and long_driver.counts.model_calls=='100016' and long_driver.selection.before=='100011')
host.args=args('terrain_lift','/long-boot-driver-B.txt',root..'/long-boot-driver.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/long-boot-driver-B.txt','rb'));local long_B=K.parse(e:read('*a'));e:close()
assert(K.validate(long_B,{},T) and K.paired(long_B,long_driver) and lift_calls==1)
boot_extra=0;boot_query=true;host.args=args('healthy','/boot-query.txt')
assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/boot-query.txt','rb'));local bad_boot=K.parse(e:read('*a'));e:close()
assert(bad_boot.verdict.status=='FAIL' and bad_boot.counts.failed=='1' and bad_boot.counts.frames=='0' and K.unhex(bad_boot.verdict.reason):find('unsupported weapon query during boot',1,true))
boot_query=false
for _,fault in ipairs({'birth_pose','fire_actual','wrong_w','same_frame'}) do
 prefix_fault=fault; same_live=fault=='same_frame'
 host.args=args('terrain_lift','/prefix-'..fault..'.txt',root..'/transport-positive.txt')
 assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
 local e=assert(io.open(root..'/prefix-'..fault..'.txt','rb'));local r=K.parse(e:read('*a'));e:close()
 assert(r.verdict.status=='FAIL' and r.counts.failed=='1' and r.counts.changed=='0' and lift_calls==0 and move_calls==0)
end
prefix_fault=nil;same_live=true;host.args=args('healthy','/same-cohort-later.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/same-cohort-later.txt','rb'));local later=e:read('*a');local r=K.parse(later);e:close()
assert(K.healthy(r,{},T) and r.selection.frame=='303' and r.selection.birth_frame=='302' and r.selection.w=='1')
host.args=args('terrain_lift','/same-cohort-later-B.txt',root..'/same-cohort-later.txt')
assert(pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/same-cohort-later-B.txt','rb'));local b=K.parse(e:read('*a'));e:close()
assert(K.validate(b,{},T) and K.paired(b,r) and lift_calls==1)
for _,mode in ipairs({'no_later','other_cohort'}) do
 no_later=mode=='no_later';other_live=mode=='other_cohort'
 host.args=args('healthy','/'..mode..'.txt')
 assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
 local e=assert(io.open(root..'/'..mode..'.txt','rb'));local r=K.parse(e:read('*a'));e:close()
 assert(r.verdict.status=='UNREACHED' and r.counts.failed=='0' and r.counts.queries==(no_later and '1' or '2') and r.counts.moving=='0')
 assert(K.unhex(r.verdict.reason)=='locked cohort lacks earlier observed eligible query')
end
no_later=false;other_live=false
same_live=false
for _,fault in ipairs({'count','eligibility','array'}) do
 reset_fault=fault;host.args=args('healthy','/reset-'..fault..'.txt')
 assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
 local e=assert(io.open(root..'/reset-'..fault..'.txt','rb'));local r=K.parse(e:read('*a'));e:close()
 assert(r.verdict.status=='FAIL' and r.counts.failed=='1' and r.counts.frames=='0' and move_calls==0)
end
reset_fault=nil
unregistered=true;host.args=args('healthy','/noncanonical.txt')
assert(not pcall(assert(loadfile(base..'/normal-fire-session.lua'))))
local e=assert(io.open(root..'/noncanonical.txt','rb'));local u=K.parse(e:read('*a'));e:close()
assert(u.verdict.status=='UNREACHED' and u.counts.failed=='0' and u.counts.births=='1' and u.counts.moving=='0')
print('normal_session_proof_gates=PASS semantic_faults=25 pre_effect_reference_faults=27 prefix_faults=4 same_cohort_later_healthy_B=2 no_later_and_other_cohort_UNREACHED=2 real_flag_store_calls='..#history..' flag_ACK_faults=3 ACK_failure_precedes_UNREACHED=3 retained_C_codec_births=1 synthetic_transport_healthy_B=2')
print('boot_retention_controls=PASS real_provider_calls=200036 first_frame_and_next_frame_queries=2 partial_feed=1 drain_faults=2 wrong_op=1 alternating_overflow=1 retired_budget=1 driver_long_boot_healthy_B=2 boot_query_refused_before_frame1=1 continuation_surfaces=6')
