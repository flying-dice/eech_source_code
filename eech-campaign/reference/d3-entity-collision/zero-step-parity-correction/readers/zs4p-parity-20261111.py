"""Whole candidate field-map parity: Python CANDIDATE_FIELDS and candidate->selection join vs the Lua K.candidate_fields and
K.validate `equal` map. Reads source text only (no import side effects); prints one JSON verdict; exit 1 on any disparity."""
import ast, json, re, sys
from pathlib import Path
W = Path(__file__).resolve().parent.parent / 'collision-characterization-20261029' / 'eech-campaign'
py = (W / 'tools/normal_fire_capture.py').read_text(encoding='utf-8')
lua = (W / 'crates/eech-world/lua/normal-fire-session-contract.lua').read_text(encoding='utf-8')
py_fields = ast.literal_eval(re.search(r"^CANDIDATE_FIELDS=('[^']*')", py, re.M).group(1)).split()
lua_fields = re.search(r"^K\.candidate_fields='([^']*)'", lua, re.M).group(1).split()
py_calls = re.findall(r"^    equal=dict\(([^)]*)\)", py, re.M); assert len(py_calls) == 1
py_join = dict(re.findall(r"(\w+)='(\w+)'", py_calls[0]))
lua_tables = re.findall(r"^ local equal=\{([^}]*)\}", lua, re.M); assert len(lua_tables) == 1
lua_join = dict(re.findall(r"(\w+)='(\w+)'", lua_tables[0]))
# The ONE deliberately non-joined candidate field: 'mode' is a literal constant ('move', the supported routed mode) checked
# separately in both validators (Python ce['mode']=='move', Lua ce.mode=='move'); the selection has no mode field. The
# candidate's test_request is checked through the actual TEST request domain, not equal to a selection field.
expected_unjoined = {'mode', 'test_request'}
mode_py = "ce['mode']=='move'" in py
mode_lua = "ce.mode=='move'" in lua
result = dict(py_fields_equal_lua_fields=py_fields == lua_fields, py_join_equal_lua_join=py_join == lua_join,
              fields=len(py_fields), joined=len(py_join), unjoined=sorted(set(py_fields) - set(py_join)),
              join_keys_not_fields=sorted(set(py_join) - set(py_fields)), mode_literal_checked_both=mode_py and mode_lua,
              schema4_joined={k: py_join.get(k) for k in ('latest', 'prestate', 'delegated')})
ok = (result['py_fields_equal_lua_fields'] and result['py_join_equal_lua_join'] and set(result['unjoined']) == expected_unjoined
      and not result['join_keys_not_fields'] and result['mode_literal_checked_both'] and result['schema4_joined'] == dict(latest='latest', prestate='prestate', delegated='delegated'))
result['verdict'] = 'PARITY' if ok else 'DISPARITY'
print(json.dumps(result))
sys.exit(0 if ok else 1)
