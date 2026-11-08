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
local lines={}
for line in report:gmatch('[^\n]+') do lines[#lines+1]=line end
local function whole(s)
    local n=tonumber(s)
    assert(n and n>=0 and n<=9007199254740991 and n==math.floor(n),'observer count outside exact integer domain')
    return n
end
local complete,bad,loads,ends,seeds,draws,pre,load,gap,digest,sites,marks,skins=lines[1]:match('^core%-crt%-summary schema=1 complete=(%d) problem=(%d) loads=(%d+) ends=(%d+) seeds=(%d+) draws=(%d+) pre=(%d+) load=(%d+) gap=(%d+) digest=(0x%x%x%x%x%x%x%x%x) sites=(%d+) marks=(%d+) skins=(%d+)$')
assert(complete=='1' and bad=='0' and loads=='1' and ends=='1' and whole(seeds)>=2,'incomplete/refused observer coverage')
assert(whole(draws)==whole(pre)+whole(load)+whole(gap),'draw totals disagree')
local count={mark=0,site=0,skin=0}
local last_order,last_draws=0,0
local boundaries={before_reinitialise=0,after_reinitialise=0,before_create=0}
for i=2,#lines do
    local family=lines[i]:match('^core%-crt%-(%a+) ')
    assert(count[family]~=nil,'unknown observer row')
    count[family]=count[family]+1
    if family=='mark' then
        local order,n=lines[i]:match(' order=(%d+) draws=(%d+) ')
        assert(order and whole(order)>last_order and whole(n)>=last_draws,'observer mark order invalid')
        last_order,last_draws=whole(order),whole(n)
        local kind=lines[i]:match('kind=([a-z_]+) ')
        if boundaries[kind]~=nil then boundaries[kind]=boundaries[kind]+1 end
    end
end
assert(count.site==whole(sites) and count.mark==whole(marks) and count.skin==whole(skins),'observer row coverage differs')
assert(lines[2]:match('kind=seed ') and lines[1+count.mark]:match('kind=seed '),'missing seed bounds')
-- Capture closes at the ACTUAL next seed. Later boot marks legitimately lie
-- outside that window; their absence must not turn an earlier reseed into a hunt.
assert(boundaries.before_reinitialise<=1 and boundaries.after_reinitialise<=boundaries.before_reinitialise and boundaries.before_create<=boundaries.after_reinitialise,'invalid boot phase prefix')
assert(out:write(report));assert(out:close())
host.log('Core CRT observer complete; diagnostic only, no loader-removal acceptance')
