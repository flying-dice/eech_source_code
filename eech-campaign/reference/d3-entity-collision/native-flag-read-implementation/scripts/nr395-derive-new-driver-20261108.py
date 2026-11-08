"""Narrow derivation of NEW witness only; old30ad is immutable."""
import pathlib,re,hashlib,json
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
p=W/'eech-campaign/crates/eech-world/lua/physical-flags-read-witness.lua';s=p.read_text()
old=W/'eech-campaign/crates/eech-world/lua/physical-flags-witness.lua'
assert hashlib.sha256(old.read_bytes()).hexdigest()=='30adcae241d43deddb710d014ac4585b3c0f282a7309b833c77c08cc71881ad7'
assert 'pf-read-identity' not in s
s=s[s.index('-- NR395 IMPLEMENTATION:'):]
s=s.replace('"flags_control"','"read_probe_control"').replace('flags_control =','read_probe_control =')
s=s.replace('pf-witness','pf-read').replace('pf%-witness','pf%-read')
# The new driver has only the accepted seven modes, not the old twenty-three.
start=s.index('local MODES = {');end=s.index('local plan = MODES [mode]',start)
s=s[:start]+'''local MODES = {
 read_probe_control = {profile="hosted"},
 read_changed = {profile="hosted",at="read_consumable",changed=true},
 read_withheld = {profile="hosted",at="read_consumable",fault="withheld",marker="entity_scan.callback"},
 read_malformed = {profile="hosted",at="read_consumable",fault="malformed",marker="entity_scan.answer"},
 read_wrong_state = {profile="hosted",at="read_consumable",fault="wrong_state",marker="entity_scan.state"},
 read_wrong_array = {profile="hosted",at="read_consumable",fault="wrong_array",marker="flag.answer"},
 read_wrong_word = {profile="hosted",at="read_consumable",fault="wrong_word",marker="flag.word"},
}
'''+s[end:]
start=s.index('local function read_reference (');end=s.index('-- a COMPLETE healthy positive:',start)
s=s[:start]+s[end:]
at=s.index('\nlocal COUNT_SCHEMA');s=s[:at]+'\nfor _, k in ipairs(Tx.extra_identity) do IDENTITY_SCHEMA[#IDENTITY_SCHEMA+1]=k end'+s[at:]
at=s.index('\nlocal FIRST_FIELDS');s=s[:at]+'\nfor _, k in ipairs({"candidate_questions","bind_calls"}) do COUNT_SCHEMA[#COUNT_SCHEMA+1]=k end'+s[at:]
start=s.index('if mode == "read_probe_control" and args.original ~= nil then');end=s.index('elseif intervening then',start)
s=s[:start]+'if intervening then'+s[end+len('elseif intervening then'):]
s=s.replace('local rec = read_reference (args.control, "control", "read_probe_control")','local rec = Tx.read_reference(args.control,args,strict_patterns,mask_pattern)')
at=s.index('local reference = {}');s=s[:at]+'Tx.configure_schema(IDENTITY_SCHEMA,COUNT_SCHEMA,ORACLE_SCHEMA,FIRST_SCHEMA,CONSUME_SCHEMA)\n'+s[at:]
needle='emit (string.format ("pf-read-identity '
at=s.index(needle);end=s.index('\n',at)
s=s[:at]+'''emit (string.format ("pf-read-identity schema=1 mode=%s profile=%s scenario=%s seed=%s frame_ms=%s hours=%s campaign=%s map=%s paired=1 runtime=%s lua=%s protocol=%s plan=%s bundle=%s reference=%s strategy=first_eligible",mode,plan.profile,scenario_name,tostring(seed),tostring(frame_ms),tostring(hours),effective_campaign,effective_map,args.runtime_code,args.lua_sha256,args.protocol_sha256,args.plan_sha256,args.bundle_sha256,args.control_sha256 or "none"))'''+s[end:]
s=s.replace('oracle_mismatches=%d",','oracle_mismatches=%d candidate_questions=%d bind_calls=%d",')
assert 'state.oracle.sets, state.oracle.reads, state.oracle.resets, state.oracle.mismatches))' in s
s=s.replace('state.oracle.sets, state.oracle.reads, state.oracle.resets, state.oracle.mismatches))','state.oracle.sets, state.oracle.reads, state.oracle.resets, state.oracle.mismatches,state.candidate_questions,state.bind_calls))',1)
s=s.replace('\n\tif state.intervention ~= nil then emit','\n\tTx.emit(state.transaction,emit)\n\tif state.intervention ~= nil then emit')
needle='if mode == "read_probe_control" then\n\t\tfor _, event'
assert needle in s
s=s.replace(needle,'''if mode == "read_probe_control" then
        if not state.transaction then return verdict("UNREACHED","no real eligible candidate in fixed window") end
        if state.transaction.phase ~= "complete" or not state.consumed then return verdict("FAILURE","selected READ transaction did not complete its exact re-ask") end
        for _, event''')
s=s.replace('if c == nil then return verdict ("UNREACHED", "the changed answer was made and accepted but its candidate question was not observed next") end','if c == nil then return verdict ("FAILURE", "changed READ did not carry into the exact next candidate") end')
p.write_text(s,encoding='utf-8',newline='\n')
with (E/'nr395-driver-derivation-20261108.json').open('x',encoding='utf-8') as f:json.dump({'old_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'new_initial_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'scope':'New family/report prefixes, seven modes, existing full positive validator/oracle/causal checks reused plus transaction module; no old source overwritten'},f,indent=2)
print('NEW driver derived; old30ad unchanged')
