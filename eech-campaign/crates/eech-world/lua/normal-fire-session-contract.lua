-- TEST-ONLY fixed-bound normal campaign capture. No engine, provider or I/O.
local K={}
K.schemas={
 identity='schema mode capture runtime bundle driver collector transaction contract reference profile scenario seed frame_ms warmup settle',
 counts='frames fires births steps queries model_calls moving failed changed',
 selection='frame birth_frame fire detach w step query before after need read carry test launcher target weapon scene name member group bind key lane word fact pose request reask position motion base ceiling previous current',
 evidence='fire detach birth step query models query_end step_end',
 verdict='status reason'
}
function K.patterns(s,n)
 if type(s)~='string' or s=='' then return nil end
 local a={};for w in (s..','):gmatch('(.-),') do if #w~=10 or not w:match('^0x[0-9a-f]+$') then return nil end;a[#a+1]=w end
 if n and #a~=n then return nil end;return a
end
function K.uint(s)
 assert(type(s)=='string' and s:match('^%d+$'),'whole number');local n=tonumber(s)
 assert(n and n<=9007199254740991 and tostring(n)==s,'canonical whole number');return n
end
function K.number(w)
 assert(K.patterns(w,1),'float word');local n=tonumber(w:sub(3),16);local sign=n>=2147483648 and -1 or 1
 local exponent=math.floor(n/8388608)%256;assert(exponent~=255,'nonfinite word');local value=n%8388608
 if exponent~=0 then value=value+8388608 end
 local power=(exponent==0 and 1 or exponent)-150
 while power>0 do value=value*2;power=power-1 end
 while power<0 do value=value/2;power=power+1 end
 return sign*value
end
function K.mask(lane) local digit=math.floor(lane/4);local bit=1;for _=1,lane%4 do bit=bit*2 end;return '0x'..string.rep('0',7-digit)..string.format('%x',bit)..string.rep('0',digit) end
function K.unhex(s)
 assert(type(s)=='string' and s:sub(1,1)=='h' and #s%2==1 and not s:sub(2):find('[^0-9a-f]'),'encoded text')
 return (s:sub(2):gsub('..',function(w)return string.char(tonumber(w,16))end))
end
function K.row(family,row)
 local out={'normal-capture',family};local keys={}
 for k in K.schemas[family]:gmatch('%S+') do assert(row[k]~=nil,'missing '..k);keys[k]=true;out[#out+1]=k..'='..tostring(row[k]) end
 for k in pairs(row) do assert(keys[k],'extra '..k) end
 return table.concat(out,' ')
end
local function tokens(body)
 local a={};for token in body:gmatch('%S+') do local k,v=token:match('^([a-z_]+)=(%S+)$');assert(k and not a[k],'field grammar');a[k]=v end;return a
end
function K.parse(text)
 assert(type(text)=='string' and #text<=262144 and text:sub(-1)=='\n' and not text:find('\r') and not text:find('\n\n'),'report framing')
 local r={};local order={'identity','counts','selection','evidence','verdict'};local at=1
 for line in text:gmatch('[^\n]+') do
  local family,body=line:match('^normal%-capture ([a-z]+) (.+)$');assert(K.schemas[family] and not r[family],'report family')
  while order[at]~=family and (order[at]=='selection' or order[at]=='evidence') do at=at+1 end
  assert(order[at]==family,'report order');at=at+1;r[family]=tokens(body);assert(K.row(family,r[family])==line,'canonical fields')
 end
 assert(r.identity and r.counts and r.verdict,'incomplete report');return r
end
function K.validate(r,expected,T)
 local a,c,s,e=r.identity,r.counts,r.selection,r.evidence
 for k,v in pairs(expected) do assert(a[k]==tostring(v),'reference identity '..k) end
 assert(a.mode=='healthy' or a.mode=='terrain_lift','mode')
 assert((a.mode=='healthy' and a.reference=='none') or (a.mode=='terrain_lift' and #a.reference==64 and a.reference:match('^[0-9a-f]+$')),'reference')
 assert(a.schema=='1' and a.profile=='reference-hosted' and a.scenario=='lebanon_retail' and a.seed=='1' and a.frame_ms=='100' and a.warmup=='30' and a.settle=='120','fixed setup')
 for _,k in ipairs({'runtime'}) do assert(#a[k]==40 and a[k]:match('^[0-9a-f]+$'),'source identity') end
 for _,k in ipairs({'bundle','driver','collector','transaction','contract'}) do assert(#a[k]==64 and a[k]:match('^[0-9a-f]+$'),'artifact identity') end
 for k,v in pairs(c) do K.uint(v) end
 assert(c.frames=='1500' and c.failed=='0' and c.changed==(a.mode=='healthy' and '0' or '1'),'healthy counts')
 assert(r.verdict.status=='PASS' and K.unhex(r.verdict.reason)==(a.mode=='healthy' and 'healthy normal cohort' or 'same-cohort terrain effect'),'positive required')
 assert(s and e,'positive witness missing')
 for _,k in ipairs({'fires','births','steps','queries','model_calls','moving'}) do assert(K.uint(c[k])>0,'missing traffic '..k) end
 for _,k in ipairs({'frame','birth_frame','fire','detach','w','step','query','before','after','need','read','carry','test','weapon','scene','member','group','bind','lane'}) do K.uint(s[k]) end
 assert(tonumber(s.birth_frame)>300 and tonumber(s.birth_frame)<=tonumber(s.frame) and tonumber(s.frame)<=1500,'warmup/window')
 assert(tonumber(s.before)<tonumber(s.need) and tonumber(s.need)<tonumber(s.read) and tonumber(s.read)+1==tonumber(s.carry) and tonumber(s.carry)<=tonumber(s.test) and tonumber(s.test)<=tonumber(s.after) and tonumber(s.after)<=tonumber(c.model_calls),'callback order')
 for _,k in ipairs({'launcher','target'}) do assert(s[k]:match('^eech%-world:[1-9]%d*$'),'canonical identity') end
 assert(s.launcher~=s.target and s.key:match('^f[1-9]%d*$') and #s.key<=20 and tonumber(s.lane)<=31,'store identity')
 assert((s.word=='0x00000000' or s.word==K.mask(tonumber(s.lane))) and (s.fact==(s.word=='0x00000000' and '0x3f800000' or '0x40000000')),'raw carried fact')
 local q=assert(T.request(tonumber(s.scene),nil,nil,nil,s.request..' '..s.target,K.patterns))
 local name=K.unhex(s.name);assert(#name<=127 and not name:find('%z'),'scene name')
 assert(not T.answer(q,{result='need_updated',name=name},{[q.scene]=name},K.patterns),'delegated positive')
 assert(T.eligible(q),'lawful request');assert(not T.same_reask({scene=q.scene,identity=q.identity,words=q.words,answered_word=s.word},{scene=q.scene,identity=q.identity,words=assert(K.patterns(s.reask,44))}),'same candidate carry')
 local pose=assert(K.patterns(s.pose,16));for _,w in ipairs(pose) do assert(math.floor(tonumber(w:sub(3),16)/8388608)%256~=255,'finite pose') end
 assert(not T.answer(q,{result='test',name=name,p=s.pose},{[q.scene]=name},K.patterns),'delegated moving pose')
 local vector={}
 for _,k in ipairs({'position','motion'}) do local a={};for v in (s[k]..','):gmatch('(.-),') do local x=tonumber(v);assert(x and x==x and math.abs(x)<math.huge,'finite motion');a[#a+1]=x end;assert(#a==3,'motion count');vector[k]=a end
 assert(vector.motion[1]^2+vector.motion[2]^2+vector.motion[3]^2>1 and K.number(pose[16])>1,'actual moving pose')
 for i=1,3 do assert(K.number(q.words[13+i])==vector.position[i] and K.number(q.words[16+i])==vector.motion[i],'same host motion/facts') end
 for _,k in ipairs({'base','ceiling','previous','current'}) do local x=tonumber(s[k]);assert(x and x==x and math.abs(x)<math.huge,'finite scalar') end
 assert(tonumber(s.base)+500<=tonumber(s.ceiling) and tonumber(s.current)>tonumber(s.previous),'physical input/clock')
 local events={};for k,v in pairs(e) do local text=K.unhex(v);assert(#text>0,'evidence missing');events[k]=text end
 local function event(k,kind)
  assert(K.event,'collector grammar missing');K.event(events[k])
  local t=tokens(assert(events[k]:match('^normal%-ev (.+)$')));assert(t.kind==kind,'evidence kind');K.uint(t.seq);K.uint(t.tick);return t
 end
 local f=event('fire','normal_fire');local d=event('detach','normal_detach');local b=event('birth','normal_birth');local st=event('step','normal_step');local qev=event('query','normal_query');local qe=event('query_end','normal_query_end');local se=event('step_end','normal_step_end')
 assert(f.fire==s.fire and d.fire==s.fire and b.fire==s.fire and d.detach==s.detach and b.detach==s.detach and b.w==s.w and b.matched=='1','fire/birth evidence')
 assert(d.launcher==s.launcher and d.weapon==s.weapon and f.ready=='1','normal producer/detachment')
 assert(K.number(f.angle)>=0 and K.number(f.angle)<=K.number(f.maximum),'actual firing angle')
 for _,x in ipairs({f,d,b}) do assert(tonumber(x.tick)==(tonumber(s.birth_frame)-1)*100,'birth frame') end
 assert(tonumber(f.seq)<tonumber(d.seq) and tonumber(d.seq)<tonumber(b.seq) and tonumber(b.seq)<tonumber(st.seq) and tonumber(st.seq)<tonumber(qev.seq) and tonumber(qev.seq)<tonumber(qe.seq) and tonumber(qe.seq)<tonumber(se.seq),'event order')
 for _,x in ipairs({d,b,st,se}) do for _,w in ipairs(assert(K.patterns(x.pose,12))) do K.number(w) end end
 for _,x in ipairs({f,b,st}) do assert(x.launcher==s.launcher and x.target==s.target and x.weapon==s.weapon,'cohort evidence') end
 for _,x in ipairs({st,qev,qe,se}) do assert(x.w==s.w and x.step==s.step and tonumber(x.tick)==(tonumber(s.frame)-1)*100,'frame evidence') end
 assert(qev.query==s.query and qe.query==s.query and qev.before==s.before and qe.after==s.after,'query bounds')
 local count=0;local calls={};for line in events.models:gmatch('[^\n]+') do K.event(line);local x=tokens(assert(line:match('^normal%-ev (.+)$')));count=count+1;assert(x.kind=='normal_model' and x.w==s.w and x.step==s.step and x.query==s.query and tonumber(x.call)==tonumber(s.before)+count and tonumber(x.tick)==(tonumber(s.frame)-1)*100,'provider interval');calls[x.call]=x.op end
 assert(count==tonumber(s.after)-tonumber(s.before) and count>0,'complete provider interval')
 assert(calls[s.need]=='16' and calls[s.read]=='17' and calls[s.carry]=='16' and calls[s.test]=='16','callback roles')
 return s
end
function K.healthy(r,expected,T) assert(r.identity.mode=='healthy','healthy first');return K.validate(r,expected,T) end
return K
