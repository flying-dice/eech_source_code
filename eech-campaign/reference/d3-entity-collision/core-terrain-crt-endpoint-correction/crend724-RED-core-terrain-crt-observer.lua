-- TEST-ONLY: delegate the unchanged campaign client, retain its real session,
-- then read the dedicated Core CRT observer. Never wraps the Host executable CRT.
local args=host.args
assert(args.scenario=='georgia_retail' or args.scenario=='lebanon_retail','fixed supported scenario required')
assert(args.hours=='0.001' and args.frame_ms=='100' and args.seed=='1','fixed hours/frame/seed required')
assert(type(args.campaign_lua)=='string' and args.campaign_lua~='','campaign_lua=<frozen campaign.lua>')
assert(type(args.crt_out)=='string' and args.crt_out~='','crt_out=<new report>')
local out=assert(host.create_exclusive(args.crt_out))
local dc=require('eech_dc')
local original=dc.boot
local engine,boots=nil,0
dc.boot=function(...)
    boots=boots+1
    assert(boots==1,'observer permits exactly one campaign boot')
    engine=original(...)
    return engine
end
local ok,problem=pcall(dofile,args.campaign_lua)
dc.boot=original
if not ok then out:close();error(problem,0) end -- no poisoned-engine query/close
assert(boots==1 and engine,'campaign produced no single real session')
local report=engine:core_terrain_crt_summary() -- feature-only method, own runtime identity
-- Parse the exact current C text, not merely family labels or field substrings.
local MAX=9007199254740991
local function integer(s,lo,hi)
    assert(type(s)=='string' and (s=='0' or s:match('^[1-9]%d*$') or s:match('^%-[1-9]%d*$')),'noncanonical observer integer')
    local n=tonumber(s)
    assert(n and n>=lo and n<=hi and n==math.floor(n),'observer integer outside domain')
    return n
end
local function add(a,b)
    assert(b<=MAX-a,'observer sum outside exact integer domain');return a+b
end
local function row(line,family,fields)
    local tokens={}
    for token in line:gmatch('%S+') do tokens[#tokens+1]=token end
    assert(table.concat(tokens,' ')==line and tokens[1]=='core-crt-'..family,'observer row grammar')
    local names={};for name in fields:gmatch('%S+') do names[#names+1]=name end
    assert(#tokens==#names+1,'observer row fields')
    local r={}
    for i,name in ipairs(names) do
        local key,value=tokens[i+1]:match('^([a-z_]+)=(.+)$')
        assert(key==name,'observer field order/name');r[name]=value
    end
    return r
end
assert(type(report)=='string' and report:sub(-1)=='\n' and not report:find('[\r%z]'),'observer text framing')
local lines={};for line in report:gmatch('(.-)\n') do assert(line~='','empty observer row');lines[#lines+1]=line end
local summary=row(lines[1] or '','summary','schema complete problem loads ends seeds draws pre load gap digest sites marks skins')
for _,k in ipairs({'schema','complete','loads','ends'}) do assert(summary[k]=='1','incomplete/refused observer coverage') end
assert(summary.problem=='0','incomplete/refused observer coverage')
assert(summary.digest:match('^0x[0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f][0-9a-f]$'),'observer digest grammar')
for _,k in ipairs({'seeds','draws','pre','load','gap','sites','marks','skins'}) do summary[k]=integer(summary[k],0,MAX) end
assert(summary.sites<=64 and summary.marks<=64 and summary.skins<=128 and summary.seeds>=2,'observer capacities/seed coverage')
assert(summary.draws==add(add(summary.pre,summary.load),summary.gap),'draw totals disagree')
assert(#lines==1+summary.marks+summary.sites+summary.skins,'observer row coverage differs')
local marks,sites,skins={},{},{}
for i=1,summary.marks do
    local m=row(lines[1+i],'mark','kind order draws a b c')
    m.order=integer(m.order,1,MAX);m.draws=integer(m.draws,0,summary.draws)
    for _,k in ipairs({'a','b','c'}) do m[k]=integer(m[k],-2147483648,2147483647) end
    assert(m.order==add(i,m.draws) and (i==1 or m.draws>=marks[i-1].draws),'observer mark ordinal/draw join')
    marks[i]=m
end
local phase,seeds,begin_load,end_load,active_ctor=0,0,nil,nil,nil
local ctors,ctor_seen={},{}
local prefix={'before_reinitialise','after_reinitialise','before_create'};local prefix_at=0
for i,m in ipairs(marks) do
    if m.kind=='seed' then
        seeds=seeds+1
        assert(m.b>0 and m.c==phase,'observer seed phase/line')
        if phase==2 then assert(i==#marks,'observer seed did not close window');phase=3 end
    elseif m.kind=='load_begin' then
        assert(phase==0 and seeds>0 and not begin_load and m.b>=0 and m.b<=4 and m.c==0,'observer load begin')
        begin_load=m;phase=1
    elseif m.kind=='load_end' then
        assert(phase==1 and not active_ctor and not end_load and m.a>=0 and m.b>=0 and m.c>=0,'observer load end')
        end_load=m;phase=2
    elseif m.kind=='ctor_begin' then
        assert(phase==1 and not active_ctor and (m.a==1 or m.a==2) and not ctor_seen[m.a] and m.b==0 and m.c==0,'observer constructor begin')
        assert(m.a==2 or not ctor_seen[2],'observer constructor order')
        active_ctor={kind=m.a,first=m,skin_calls=0,skin_draws=0};ctor_seen[m.a]=active_ctor
    elseif m.kind=='ctor_end' then
        assert(phase==1 and active_ctor and m.a==active_ctor.kind and m.b>=-1 and m.c==0,'observer constructor end')
        active_ctor.last=m;active_ctor.scene=m.b;ctors[#ctors+1]=active_ctor;active_ctor=nil
    else
        assert(phase==2 and prefix[prefix_at+1]==m.kind and m.a==0 and m.b==0 and m.c==0,'observer boot boundary prefix/unknown kind')
        prefix_at=prefix_at+1
    end
end
assert(#marks>0 and marks[1].kind=='seed' and phase==3 and begin_load and end_load and not active_ctor and ctor_seen[2],'observer load/seed/constructor bounds')
assert(seeds==summary.seeds and begin_load.draws==summary.pre and end_load.draws==add(summary.pre,summary.load) and marks[#marks].draws==summary.draws,'observer summary/mark phases')
local endpoints,site_keys={},{}
local phase_first={0,begin_load.order,end_load.order}
local phase_end={begin_load.order,end_load.order,marks[#marks].order}
local phase_draw_end={summary.pre,add(summary.pre,summary.load),summary.draws}
local totals={0,0,0};local phase_names={'pre','load','gap'}
local function draw_rank(ordinal)
    local preceding=0
    for _,m in ipairs(marks) do assert(m.order~=ordinal,'site ordinal names a mark');if m.order<ordinal then preceding=preceding+1 end end
    return ordinal-preceding
end
for i=1,summary.sites do
    local s=row(lines[1+summary.marks+i],'site','file line pre load gap first_pre first_load first_gap last first_value last_value gap_scene')
    assert(s.file:match('^[%w_.%-]+$') and s.file~='.' and s.file~='..','observer site basename')
    s.line=integer(s.line,1,2147483647)
    local key=s.file..':'..s.line;assert(not site_keys[key],'duplicate observer site');site_keys[key]=true
    for _,k in ipairs({'pre','load','gap','first_pre','first_load','first_gap','last'}) do s[k]=integer(s[k],0,MAX) end
    s.first_value=integer(s.first_value,0,2147483647);s.last_value=integer(s.last_value,0,2147483647);s.gap_scene=integer(s.gap_scene,-1,2147483647)
    local count,latest,latest_count=0,0,0
    for p,name in ipairs(phase_names) do
        local n,first=s[name],s['first_'..name];totals[p]=add(totals[p],n);count=add(count,n)
        assert((n==0 and first==0) or (n>0 and first>phase_first[p] and first<phase_end[p]),'observer site phase ordinal')
        if n>0 then
            assert(not endpoints[first],'duplicate site draw ordinal');endpoints[first]=key
            local room=phase_draw_end[p]-draw_rank(first)+1
            assert(n<=room,'observer site count exceeds phase ordinal window')
            latest=first;latest_count=n
        end
    end
    if count==0 then
        assert(s.last==0 and s.first_value==0 and s.last_value==0 and s.gap_scene==0,'observer empty site fields')
    else
        assert(s.last>=latest and s.last<marks[#marks].order and (count~=1 or s.last==latest),'observer site last ordinal')
        local last_phase=s.gap>0 and 3 or (s.load>0 and 2 or 1)
        assert(s.last<phase_end[last_phase] and draw_rank(s.last)-draw_rank(latest)+1>=latest_count,'observer site last phase/count')
        assert(not endpoints[s.last] or endpoints[s.last]==key,'duplicate site last ordinal');endpoints[s.last]=key
        if count==1 then assert(s.first_value==s.last_value,'observer one-draw values') end
        if s.gap==0 then assert(s.gap_scene==0,'observer absent gap scene') end
    end
    sites[i]=s
end
assert(totals[1]==summary.pre and totals[2]==summary.load and totals[3]==summary.gap,'observer summary/site phases')
-- Every mark's draw prefix must fit the per-site first/count/last evidence.
-- These are necessary bounds on the compact report, not a reconstructed trace.
for _,m in ipairs(marks) do
    local minimum,maximum=0,0
    for _,s in ipairs(sites) do
        for p,name in ipairs(phase_names) do
            local n,first=s[name],s['first_'..name]
            local lo,hi=0,0
            if n>0 and first<m.order then
                if m.order>=phase_end[p] then lo=n;hi=n
                else
                    lo=math.max(1,n-(phase_draw_end[p]-m.draws))
                    hi=math.min(n,m.draws-draw_rank(first)+1)
                    local last_phase=s.gap>0 and 3 or (s.load>0 and 2 or 1)
                    if p==last_phase then if s.last<m.order then lo=n else hi=math.min(hi,n-1) end end
                end
            end
            assert(lo<=hi,'observer site/mark prefix bounds')
            minimum=add(minimum,lo);maximum=add(maximum,hi)
        end
    end
    assert(m.draws>=minimum and m.draws<=maximum,'observer site/mark draw prefix')
end
for _,m in ipairs(marks) do
    if m.kind=='seed' then local found=false;for _,s in ipairs(sites) do if s.line==m.b then found=true end end;assert(found,'observer seed site missing') end
end
local skin_keys,skin_draws={},0
for i=1,summary.skins do
    local s=row(lines[1+summary.marks+summary.sites+i],'skin','scene animation season normal desert winter ctor frames calls draws')
    s.scene=integer(s.scene,0,2147483647);s.animation=integer(s.animation,-2147483648,2147483647);s.season=integer(s.season,0,4)
    for _,k in ipairs({'normal','desert','winter'}) do s[k]=integer(s[k],0,2147483647) end
    s.ctor=integer(s.ctor,0,2);s.frames=integer(s.frames,-1,2147483647);s.calls=integer(s.calls,1,MAX);s.draws=integer(s.draws,0,MAX)
    local key=s.scene..':'..s.animation..':'..s.season..':'..s.ctor;assert(not skin_keys[key],'duplicate observer skin');skin_keys[key]=true
    assert((s.animation<0 and s.frames==-1 and s.draws==0) or (s.animation>=0 and s.frames>=0 and s.draws==s.calls),'observer skin activation/draws')
    if s.ctor>0 then
        local ctor=ctor_seen[s.ctor]
        assert(ctor and ctor.last and ctor.scene==s.scene and s.season==begin_load.b,'observer constructor skin identity')
        ctor.skin_calls=add(ctor.skin_calls,s.calls);ctor.skin_draws=add(ctor.skin_draws,s.draws)
    end
    skin_draws=add(skin_draws,s.draws);skins[i]=s
end
assert(skin_draws<=summary.draws,'observer skin/global draws')
for _,ctor in ipairs(ctors) do
    assert(ctor.skin_calls==(ctor.scene<0 and 0 or 1) and ctor.skin_draws<=ctor.last.draws-ctor.first.draws,'observer constructor skin coverage')
end
for _,s in ipairs(sites) do
    if s.gap>0 and s.gap_scene>=0 then
        local found=false;for _,skin in ipairs(skins) do if skin.ctor==0 and skin.scene==s.gap_scene and skin.draws>0 then found=true end end
        assert(found,'observer gap skin scene missing')
    end
end
-- Counts/endpoints cannot reconstruct a per-draw trace or identify every value
-- consumer. Later boot marks legitimately disappear at an earlier closing seed.
assert(out:write(report));assert(out:close())
host.log('Core CRT observer complete; diagnostic only, no loader-removal acceptance')
