import datetime,json,hashlib
from pathlib import Path
w=Path('R:/d2-evidence/weapon-characterization-20261027')
def summary(name):
    rows=[json.loads(s) for s in (w/f'out/{name}/summary.jsonl').read_text().splitlines() if s.strip()]
    assert len(rows)==1 and rows[0]['event']=='summary' and rows[0]['failure']==''
    return rows[0]
reference=summary('h1-pref');altered=summary('h1-pnever')
assert reference['mode']=='reference' and altered['mode']=='reacquire_never'
assert reference['frames']==altered['frames']==6001
assert reference['reacquire_asked']==reference['reacquire_returned']==57
assert altered['reacquire_asked']==23 and altered['reacquire_returned']==0
assert reference['births_done']!=altered['births_done'] and reference['losses']!=altered['losses']
for name in ('pref','pnever'):
    assert 'inputs IDENTICAL' in (w/f'root-h1-{name}-post-input.log').read_text(encoding='utf-8-sig')
record=dict(when_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),source='a137fc3ec1963d88de5f0b8562145b358b50e111',
    scope='Matched600s/seed1/100ms Lebanon public-host causal pair, same clean a137 fixture; no fullpack equivalence claim',
    actual_native_exits={'h1-pref':0,'h1-pnever':0},wall_seconds={'h1-pref':1139,'h1-pnever':1151},
    completed_utc={'h1-pref':'2026-10-28T02:14:10Z','h1-pnever':'2026-10-28T02:33:21Z'},
    host_change='Provider delegates to reference; only a DONE weapon_reacquire action flag is changed to0. No additional random draw.',
    reference=reference,altered=altered,post_input_actual_exits={'h1-pref':0,'h1-pnever':0},
    all35_exports_and_BUILD_INFO_unchanged=True,
    limitations='Other provider modes use180s and must NOT be compared directly to this600s summary as an isolated causal pair. No own-model seduction/reacquisition activity claimed. Remaining malformed cohort/fullpack/review/CI/merge45 pending.')
(w/'root-h1-reacquisition-causal-completion.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print('Matched600s causal pair: reacquisition57/57 ->23/0; actual births1700->1109, losses50->47; both native0/post1179inputs0.')
