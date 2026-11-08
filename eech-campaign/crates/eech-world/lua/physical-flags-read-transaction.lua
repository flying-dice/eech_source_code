-- TEST-ONLY transaction checks for the nr395 opt-in public flag READ witness.
-- No model/store answers are implemented here. The real reference provider is
-- called by the driver, and only already-accepted boot scene names are retained.
local T = {}
T.modes = {read_probe_control=true,read_changed=true,read_withheld=true,read_malformed=true,read_wrong_state=true,read_wrong_array=true,read_wrong_word=true}
T.extra_identity = {'runtime','lua','protocol','plan','bundle','reference','strategy'}
T.selection_schema = {'tag','scene','id','name','member','group','bind','candidate','frame','entry','original','pose','request','group_opened','group_acked','key','array','lane','ordinal','before_group','before_set','before_read','before_reset','words'}
T.store_schema = {'key','array','lane','before','mask','real','answered','after'}
T.reask_schema = {'words','fact'}
local ZERO, NEGZERO, ONE, TWO = '0x00000000', '0x80000000', '0x3f800000', '0x40000000'
local function zero(w) return w == ZERO or w == NEGZERO end
local function uint(n) return type(n) == 'number' and n >= 1 and n <= 9007199254740991 and n == math.floor(n) end
local function finite(w) return math.floor(tonumber(w:sub(3), 16) / 8388608) % 256 ~= 255 end
function T.hex(s)
	assert(type(s) == 'string')
	return 'h' .. (s:gsub('.', function(c) return string.format('%02x', string.byte(c)) end))
end
function T.request(scene, depth, kind, definition, key, patterns)
	if (not uint(scene) and scene ~= 0) or scene > 2147483647 then return nil, 'candidate scene is not a whole nonnegative native index' end
	if depth ~= nil or kind ~= nil or definition ~= nil then return nil, 'candidate has depth/kind/definition' end
	if type(key) ~= 'string' then return nil, 'candidate has no raw payload' end
	local head, rest = key:match('^(%S+)(.*)$')
	local words = head and patterns(head, 44)
	if not words then return nil, 'candidate is not exactly44 raw words' end
	local identity = rest == '' and '' or rest:match('^ (.+)$')
	if identity == nil or #identity > 128 or identity:find('%z') then return nil, 'candidate identity is outside its domain' end
	for i, w in ipairs(words) do if i ~= 13 and not finite(w) then return nil, 'candidate nonfinite fact ' .. (i - 1) end end
	for _, i in ipairs({7,8,9,10,11,12,23,24,43,44}) do
		if not zero(words[i]) and words[i] ~= ONE then return nil, 'candidate invalid flag ' .. (i - 1) end
	end
	if not zero(words[21]) and words[21] ~= ONE and words[21] ~= TWO then return nil, 'candidate invalid UPDATED state' end
	if words[9] == ONE and (words[10] == ONE or words[11] == ONE) then return nil, 'candidate incompatible city roles' end
	if words[9] == ONE and scene ~= 0 then return nil, 'a city has a nonzero scene' end
	return {scene=scene, identity=identity, words=words}
end

function T.answer(q, a, scenes, patterns)
	if type(a) ~= 'table' then return 'candidate answer is not a table' end
	if getmetatable(a) ~= nil then return 'candidate delegated answer has a metatable' end
	if type(a.result) ~= 'string' then return 'candidate answer has no result' end
	for k in pairs(a) do if k ~= 'result' and k ~= 'name' and k ~= 'p' and k ~= 'reason' then return 'candidate answer has an unknown field' end end
	if a.result == 'refused' then return 'candidate provider refused: ' .. tostring(a.reason) end
	if type(a.name) ~= 'string' or #a.name > 127 or a.name:find('%z') then return 'candidate answer has invalid name' end
	if q.scene > 0 and (scenes[q.scene] == nil or a.name ~= scenes[q.scene]) then return 'candidate name differs from its C-accepted boot scene' end
	if q.scene == 0 and a.name ~= '' then return 'scene0 candidate has a name' end
	local w, r = q.words, a.result
	if r == 'test' then
		local p = patterns(a.p)
		if not p or (#p ~= 12 and #p ~= 16) or q.scene == 0 then return 'candidate test has invalid scene/pose count' end
		if #p == 16 and zero(w[7]) then return 'moving pose for nonmobile' end
		for _, word in ipairs(p) do if not finite(word) then return 'candidate test has nonfinite pose' end end
	elseif a.p ~= nil then return 'non-test candidate carries a pose'
	elseif r == 'need_updated' then
		if zero(w[7]) or not zero(w[11]) or not zero(w[21]) then return 'illegal delegated need_updated' end
	elseif r == 'need_fixed' then
		if not zero(w[7]) or not zero(w[11]) or not zero(w[43]) then return 'illegal delegated need_fixed' end
	elseif r == 'need_placement' then
		local fixed = zero(w[7]) and zero(w[11]) and not zero(w[43]) and not zero(w[8]) and zero(w[9]) and zero(w[10])
		if not zero(w[44]) or q.scene == 0 or not (not zero(w[7]) or not zero(w[11]) or fixed) then return 'illegal delegated need_placement' end
	elseif r == 'need_normal' then
		if zero(w[23]) or not zero(w[24]) or not zero(w[7]) or zero(w[8]) or zero(w[44]) then return 'illegal delegated need_normal' end
	elseif r == 'buildings' then
		if zero(w[9]) or zero(w[12]) then return 'illegal delegated buildings' end
	elseif r ~= 'skip' then return 'unknown delegated candidate result' end
	return nil
end

function T.registration(records, descriptor, identity, ordinal)
	if type(descriptor) ~= 'table' or not uint(descriptor.member_key) or not uint(descriptor.group_key) then return nil, 'invalid actual registration tuple' end
	if type(identity) ~= 'string' or identity == '' or #identity > 128 or identity:find('%z') or records[identity] then return nil, 'invalid/reused actual host identity' end
	local r = {member=descriptor.member_key, group=descriptor.group_key, bind=ordinal}
	records[identity] = r
	return r
end

function T.eligible(q) return q.scene > 0 and q.identity ~= '' and q.words[7] == ONE and zero(q.words[11]) and zero(q.words[21]) end
function T.same_reask(tx, q)
	if q.scene ~= tx.scene or q.identity ~= tx.identity then return 'candidate identity/scene changed on re-ask' end
	for i=1,44 do if i ~= 21 and q.words[i] ~= tx.words[i] then return 'candidate re-ask changed fact ' .. (i-1) end end
	local expected = tx.answered_word == ZERO and ONE or TWO
	if q.words[21] ~= expected then return 'candidate did not carry the answered raw word' end
	return nil
end

-- The oracle is a transcript-derived observation, not another physical store.
function T.oracle_word(lanes)
	local d = {}
	for digit=0,7 do
		local n, power = 0, 1
		for bit=0,3 do if lanes[digit*4+bit] then n=n+power end power=power*2 end
		d[8-digit] = string.format('%x', n)
	end
	return '0x' .. table.concat(d)
end

function T.arguments(a)
	local allowed = {root=true,out=true,mode=true,protocol=true,request_updated=true,runtime_code=true,lua_sha256=true,protocol_sha256=true,plan_sha256=true,bundle_sha256=true,control_sha256=true,control=true,scenario=true,hours=true,frame_ms=true,seed=true}
	for k in pairs(a) do assert(allowed[k], 'unknown READ witness argument '..tostring(k)) end
	assert(T.modes[a.mode], 'unknown READ witness mode')
	assert(a.request_updated == 'first_eligible', 'request_updated=first_eligible is required')
	assert(a.runtime_code == '3abf03e5133279ae6ed53b657e76d4f7724f7dd3', 'wrong runtime identity')
	assert(a.plan_sha256 == '1c472eec613c09041491617d692c91b957568a21d18d9137c316ea1cfebe7352', 'wrong accepted plan identity')
	for _, k in ipairs({'lua_sha256','protocol_sha256','bundle_sha256'}) do assert(type(a[k])=='string' and #a[k]==64 and not a[k]:find('[^0-9a-f]'), 'invalid '..k) end
	assert(a.scenario == 'lebanon_retail' and a.hours == '0.001' and a.frame_ms == '100' and a.seed == '1', 'the fixed READ setup is required')
	if a.mode == 'read_probe_control' then assert(a.control == nil and a.control_sha256 == nil, 'control takes no reference')
	else assert(type(a.control)=='string' and a.control~='' and type(a.control_sha256)=='string' and #a.control_sha256==64 and not a.control_sha256:find('[^0-9a-f]'), 'child needs exact control identity') end
end

function T.unhex(s)
	assert(type(s)=='string' and s:sub(1,1)=='h' and (#s-1)%2==0 and not s:sub(2):find('[^0-9a-f]'), 'noncanonical encoded text')
	local out = (s:sub(2):gsub('..',function(pair) return string.char(tonumber(pair,16)) end))
	assert(T.hex(out)==s, 'noncanonical text')
	return out
end
local function number(s)
	assert(type(s)=='string' and (s=='0' or s:match('^[1-9][0-9]*$')), 'noncanonical whole number')
	local n=tonumber(s); assert(n and n<=9007199254740991 and n==math.floor(n), 'whole number outside exact domain')
	return n
end
function T.configure_schema(identity,counts,oracle,first,consume)
	T.schemas={['pf-read-identity']=identity,['pf-read-counts']=counts,['pf-read-oracle']=oracle,['pf-read-first']=first,['pf-read-consume']=consume,['pf-read-selection']=T.selection_schema,['pf-read-store']=T.store_schema,['pf-read-reask']=T.reask_schema}
	T.ranks={['pf-read-identity']=1,['pf-read-counts']=2,['pf-read-oracle']=3,['pf-read-first']=4,['pf-read-consume']=5,['pf-read-selection']=6,['pf-read-store']=7,['pf-read-reask']=8,['pf-read-verdict']=9}
end
function T.read_reference(path,a,patterns,mask_pattern)
	local f=assert(io.open(path,'rb'),'cannot read healthy reference')
	local raw=f:read(131073) or '';f:close();assert(#raw<=131072,'oversized healthy reference')
	assert(raw:sub(-1)=='\n' and not raw:find('\r') and not raw:find('\n\n',1,true), 'noncanonical healthy file framing')
	local rec,rank,first,last={},0,nil,nil
	for line in raw:gmatch('([^\n]+)\n') do
		local name=line:match('^(%S+)');local r=T.ranks[name];assert(r and r>=rank,'unknown/out-of-order healthy record')
		rank=r;first=first or name;last=name
		local fields={}
		if name=='pf-read-verdict' then
			local m,s,d=line:match('^pf%-read%-verdict mode=(%S+) status=(%S+) detail=(.+)$');assert(m and s=='PASS' and m=='read_probe_control','not a healthy verdict')
			fields={mode=m,status=s,detail=d}
		else
			local tokens={};for token in line:gmatch('%S+') do tokens[#tokens+1]=token end
			local schema=T.schemas[name];assert(#tokens==#schema+1,'healthy field count')
			for i,k in ipairs(schema) do local key,value=tokens[i+1]:match('^([^=]+)=(.+)$');assert(key==k and not fields[key],'noncanonical healthy field order/key');fields[key]=value end
			local canonical={name};for _,k in ipairs(schema) do canonical[#canonical+1]=k..'='..fields[k] end
			assert(table.concat(canonical,' ')==line,'noncanonical healthy whitespace')
		end
		if name=='pf-read-first' then rec[name]=rec[name] or {};rec[name][#rec[name]+1]=fields
		else assert(rec[name]==nil,'duplicate healthy record');rec[name]=fields end
	end
	assert(first=='pf-read-identity' and last=='pf-read-verdict','healthy file endpoints')
	for _,name in ipairs({'pf-read-counts','pf-read-oracle','pf-read-consume','pf-read-selection','pf-read-store','pf-read-reask'}) do assert(rec[name],'incomplete healthy '..name) end
	local id=rec['pf-read-identity']
	local expected={schema='1',mode='read_probe_control',profile='hosted',scenario='lebanon_retail',seed='1',frame_ms='100',hours='0.001',campaign='LEBANON.CHC',map='..\\common\\maps\\map5',paired='1',runtime=a.runtime_code,lua=a.lua_sha256,protocol=a.protocol_sha256,plan=a.plan_sha256,bundle=a.bundle_sha256,reference='none',strategy='first_eligible'}
	for k,v in pairs(expected) do assert(id[k]==v,'healthy identity mismatch '..k) end
	local s,store,reask=rec['pf-read-selection'],rec['pf-read-store'],rec['pf-read-reask']
	assert(s.tag=='1' and s.array=='0' and s.group_opened==s.group_acked and (s.group_opened=='0' or s.group_opened=='1'),'healthy selection/ACK')
	for _,k in ipairs({'scene','member','group','bind','candidate','ordinal'}) do assert(number(s[k])>0,'healthy positive correlation '..k) end
	for _,k in ipairs({'frame','lane','before_group','before_set','before_read','before_reset'}) do number(s[k]) end
	assert(number(s.lane)<=31 and number(s.frame)<=35,'healthy bounded selection')
	local entry=T.unhex(s.entry);assert(entry==(s.frame=='0' and 'boot' or 'frame:'..s.frame),'healthy entry/frame')
	local idtext=T.unhex(s.id);assert(idtext~='' and #idtext<=128 and not idtext:find('%z'),'healthy selected identity')
	local q,why=T.request(number(s.scene),nil,nil,nil,s.words..' '..idtext,patterns);assert(q and T.eligible(q),why or 'healthy selection is not eligible')
	assert(table.concat(q.words,',')==s.words,'noncanonical healthy request patterns')
	assert(T.unhex(s.name)~='' and #T.unhex(s.name)<=127 and not T.unhex(s.name):find('%z'),'healthy scene name')
	assert(s.original=='skip' or s.original=='need_updated' or s.original=='need_placement' or s.original=='test','healthy mobile delegated state')
	local answer={result=s.original,name=T.unhex(s.name),p=s.pose~='none' and s.pose or nil}
	assert(T.answer(q,answer,{[q.scene]=answer.name},patterns)==nil,'invalid complete healthy delegated answer')
	assert(s.request==(s.original=='need_updated' and 'natural' or 'diagnostic'),'healthy request role')
	assert(store.key==s.key and store.array==s.array and store.lane==s.lane and s.key:match('^f[1-9][0-9]*$') and #s.key<=20,'healthy READ correlation')
	local mask=mask_pattern(number(s.lane));assert(store.mask==mask and patterns(store.before,1),'healthy oracle word/mask')
	local w=tonumber(store.before:sub(3),16);local lane=number(s.lane);local power=1;for _=1,lane do power=power*2 end
	local real=(math.floor(w/power)%2==1) and mask or ZERO
	assert(store.real==real and store.answered==real and store.after==real,'healthy W&M relation')
	local after=patterns(reask.words,44);assert(after,'healthy reask44')
	assert(table.concat(after,',')==reask.words,'noncanonical healthy reask patterns')
	for i=1,44 do if i~=21 then assert(after[i]==q.words[i],'healthy reask changed another fact') end end
	local fact=real==ZERO and ONE or TWO;assert(after[21]==fact and reask.fact==fact,'healthy carried fact')
	local consume=rec['pf-read-consume'];assert(consume.fact==fact and consume.expected==fact and consume.host==fact and consume.real_word==real and consume.answered_word==real and consume.changed=='0','healthy consume tuple differs')
	local counts=rec['pf-read-counts'];assert(number(s.frame)<=number(counts.frames) and number(s.bind)<=number(counts.bind_calls) and number(s.candidate)<=number(counts.candidate_questions),'healthy correlation outside counts')
	assert(number(counts.frames)>=1 and number(counts.frames)<=35,'healthy outside fixed frame bound')
	local found
	for _,row in ipairs(rec['pf-read-first'] or {}) do if row.event=='read_consumable' then found=row end end
	assert(found,'healthy selected READ first row absent')
	for _,k in ipairs({'ordinal','frame','before_group','before_set','before_read','before_reset'}) do assert(found[k]==s[k],'healthy selected READ differs from first row') end
	T.reference=rec
	return rec
end

function T.match_selection(tx,reference,stage)
	if not reference then return nil end
	local r=reference['pf-read-selection']
	local values={tag='1',scene=tostring(tx.scene),id=T.hex(tx.identity),name=T.hex(tx.name),member=tostring(tx.member),group=tostring(tx.group),bind=tostring(tx.bind),candidate=tostring(tx.candidate),frame=tostring(tx.frame),entry=T.hex(tx.entry),original=tx.original,pose=tx.pose or 'none',request=tx.request,words=table.concat(tx.words,',')}
	if stage=='read' then
		for _,k in ipairs({'group_opened','group_acked','key','array','lane','ordinal','before_group','before_set','before_read','before_reset'}) do values[k]=tostring(tx[k]) end
		if tx.before_word~=reference['pf-read-store'].before then return 'selected store preword differs from healthy' end
	end
	for k,v in pairs(values) do if r[k]~=v then return 'selected healthy trajectory mismatch '..k end end
	return nil
end

function T.emit(tx,emit)
	if not tx then return end
	local values={tag='1',scene=tostring(tx.scene),id=T.hex(tx.identity),name=T.hex(tx.name),member=tostring(tx.member),group=tostring(tx.group),bind=tostring(tx.bind),candidate=tostring(tx.candidate),frame=tostring(tx.frame),entry=T.hex(tx.entry),original=tx.original,pose=tx.pose or 'none',request=tx.request,words=table.concat(tx.words,',')}
	for _,k in ipairs({'group_opened','group_acked','key','array','lane','ordinal','before_group','before_set','before_read','before_reset'}) do values[k]=tostring(tx[k] or 'none') end
	local line={'pf-read-selection'};for _,k in ipairs(T.selection_schema) do line[#line+1]=k..'='..values[k] end;emit(table.concat(line,' '))
	if tx.real_word then emit(string.format('pf-read-store key=%s array=0 lane=%d before=%s mask=%s real=%s answered=%s after=%s',tx.key,tx.lane,tx.before_word,tx.mask,tx.real_word,tx.answered_word or 'none',tx.real_word)) end
	if tx.reask then emit('pf-read-reask words='..table.concat(tx.reask,',')..' fact='..tx.reask[21]) end
end
return T
