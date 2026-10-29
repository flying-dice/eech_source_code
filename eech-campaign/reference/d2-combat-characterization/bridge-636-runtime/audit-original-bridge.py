"""Strict structural audit of the original bridge witness; never a second physics solver."""
import argparse, collections, hashlib, json, math, pathlib, sys

def require(ok, message):
    if not ok:
        raise ValueError(message)

def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'duplicate JSON key: ' + key)
        result[key] = value
    return result

def finite(value):
    if isinstance(value, float):
        require(math.isfinite(value), 'nonfinite JSON number')
    elif isinstance(value, dict):
        for item in value.values(): finite(item)
    elif isinstance(value, list):
        for item in value: finite(item)

def invalid_constant(value):
    raise ValueError('invalid JSON constant ' + value)

def audit(trace, run):
    maximum_bytes = 16 * 1024 * 1024
    with trace.open('rb') as stream:
        raw = stream.read(maximum_bytes + 1)
    require(0 < len(raw) <= maximum_bytes, 'trace size outside bounded audit')
    rows = []
    for number, line in enumerate(raw.splitlines(), 1):
        require(0 < len(line) <= 4 * 1024 * 1024, 'invalid line size at ' + str(number))
        row = json.loads(line, object_pairs_hook=unique, parse_constant=invalid_constant)
        require(isinstance(row, dict) and isinstance(row.get('event'), str), 'missing event at line ' + str(number))
        finite(row)
        rows.append(row)
    report_events = {'cmb-hash','cmb-integrity','cmb-capability','cmb-damage','cmb-explosion','cmb-blast','cmb-install','cmb-repair','cmb-log','cmb-warhead','cmb-timed','cmb-kind','cmb-blast-kill','cmb-unobserved'}
    report_scopes = {'bridge:trace:before','bridge:trace:after','run'}
    state_scopes = {'bridge:before','bridge:after'} | {'frame:' + str(n) + ':state' for n in range(1, 121)}
    log_scopes = {'pre:warmup','pre:failure'} | {'frame:' + str(n) + ':log' for n in range(121)}
    for row in rows:
        event = row['event']
        if event in report_events:
            require(row.get('scope') in report_scopes, 'unexpected report scope')
        elif event in {'cmb-bridge-state','cmb-bridge-segment'}:
            require(row.get('scope') in state_scopes, 'unexpected state scope')
        elif event in {'cmb-bridge-census','cmb-bridge-survey'}:
            require(row.get('scope') == 'bridge:survey', 'unexpected survey scope')
        elif event == 'cmb-bridge-disarm':
            require(row.get('scope') == 'bridge:disarm', 'unexpected disarm scope')
        elif event == 'cmb-ev':
            require(row.get('scope') in log_scopes, 'unexpected event-log scope')
        else:
            require(event in {'boot','bridge_case','bridge_summary'} and 'scope' not in row, 'unknown or misplaced event: ' + event)
    record = json.loads(run.read_text(encoding='utf-8-sig'), object_pairs_hook=unique)
    require(record.get('status') == 'completed' and record.get('actual_native_exit') == 0, 'native completion/exit not established')
    def one(event):
        matches = [row for row in rows if row['event'] == event]
        require(len(matches) == 1, event + ' cardinality')
        return matches[0]
    boot, case, summary = one('boot'), one('bridge_case'), one('bridge_summary')
    require(rows[0] == boot and boot.get('case') == 'bridge_collapse', 'wrong boot')
    require((boot.get('seed'), boot.get('warmup'), boot.get('frame_ms'), boot.get('window_seconds'), boot.get('criterion')) == (1, 5, 100, 12, 0), 'unexpected original witness configuration')
    require(case.get('done') is True and case.get('first_collapse') is True, 'first-collapse precondition was not reached')
    require(case.get('scenario') == boot.get('scenario') == record.get('scenario'), 'scenario mismatch')
    facts = dict(word.split('=', 1) for word in case['text'].split() if '=' in word)
    require(all(facts.get(key) == '1' for key in ['bridge_alive', 'neighbour_alive', 'first_collapse']), 'actual alive first-collapse facts missing')
    require(facts.get('precondition') == 'declared_direct_structural_failure' and facts.get('api') == 'kill_client_server_entity', 'wrong failure precondition')
    bridge = case['bridge']
    census = one('cmb-bridge-census')
    survey = [row for row in rows if row['event'] == 'cmb-bridge-survey']
    require(type(census.get('bridges')) is int and 0 < census['bridges'] <= len(rows) and len(survey) == census['bridges'], 'survey cardinality')
    require([row.get('bridge') for row in survey] == list(range(len(survey))), 'survey ordering')
    require(rows.index(census) < rows.index(survey[0]), 'survey precedes census')
    require(any(row.get('bridge') == bridge for row in survey), 'chosen bridge not surveyed')
    expected_scopes = ['bridge:before', 'bridge:after'] + ['frame:' + str(n) + ':state' for n in range(1, 121)]
    heads = [row for row in rows if row['event'] == 'cmb-bridge-state']
    require([row.get('scope') for row in heads] == expected_scopes, 'missing, duplicate or reordered state group')
    total = heads[0]['segments']
    require(isinstance(total, int) and total > 0 and all(row.get('segments') == total and row.get('bridge') == bridge for row in heads), 'bridge/segment cardinality changed')
    segments = [row for row in rows if row['event'] == 'cmb-bridge-segment']
    require(len(segments) == len(heads) * total, 'missing or extra segment row')
    for head in heads:
        group = [row for row in segments if row.get('scope') == head['scope']]
        require([row.get('index') for row in group] == list(range(total)), 'missing, duplicate or reordered segment in ' + head['scope'])
        at = rows.index(head)
        require(rows[at + 1:at + 1 + total] == group, 'segment rows not adjacent to their state header')
        require(all(row.get('bridge') == bridge and row.get('at') == head.get('at') and row.get('alive') in (0, 1) and row.get('in_update') in (0, 1) for row in group), 'inconsistent segment snapshot')
    require(heads[0]['alive'] == 1 and heads[1]['alive'] == heads[-1]['alive'] == 0, 'parent liveness not first-loss transition')
    require(heads[1]['breaks'] == heads[0]['breaks'] + 1 and all(row['breaks'] == heads[1]['breaks'] for row in heads[1:]), 'route break repeated or absent')
    require(heads[0]['at'] == heads[1]['at'] and all(heads[i]['at'] > heads[i-1]['at'] for i in range(2, len(heads))), 'frame clock did not advance or immediate state mutated time')
    # Empty event drains are valid; a present row must remain in its script phase.
    phases = ['boot','pre:warmup','bridge:survey','bridge:trace:before','bridge:before','pre:failure','bridge_case','bridge:after','frame:0:log']
    for frame in range(1, 121):
        phases.extend(['frame:' + str(frame) + ':state', 'frame:' + str(frame) + ':log'])
    phases.extend(['bridge_summary','bridge:disarm','bridge:trace:after','run'])
    phase_index = {name: index for index, name in enumerate(phases)}
    positions = [phase_index[row.get('scope', row['event'])] for row in rows]
    require(all(left <= right for left, right in zip(positions, positions[1:])), 'marker/report/state/log phase ordering')
    state_times = {head['scope']: head['at'] for head in heads}
    before_time, final_time = heads[0]['at'], heads[-1]['at']
    early_scopes = {'pre:warmup','bridge:survey','bridge:trace:before','bridge:before','pre:failure','bridge:after','frame:0:log'}
    final_scopes = {'bridge:disarm','bridge:trace:after','run'}
    for row in rows:
        scope = row.get('scope')
        if scope is None:
            continue
        if scope in early_scopes:
            expected_time = before_time
        elif scope in final_scopes:
            expected_time = final_time
        else:
            frame = int(scope.split(':')[1])
            expected_time = state_times['frame:' + str(frame) + ':state']
        require(row.get('at') == expected_time, 'row time disagrees with its matching state/phase: ' + scope)
    events = [row for row in rows if row['event'] == 'cmb-ev']
    require([row.get('seq') for row in events] == list(range(1, len(events) + 1)), 'event sequence gap/duplicate/reordering')
    bridge_events = [row for row in events if row.get('kind', '').startswith('bridge-')]
    kinds = {'collision','shape','child-killed','link-broken','repair-task','update-inserted','segment-update','unsupported-kill','supported-removed','fall','landed'}
    require(all(row['kind'][7:] in kinds and row.get('bridge') == bridge and row.get('of') == total and 0 <= row.get('seg', -1) < total for row in bridge_events), 'unknown or inconsistent bridge event')
    counts = collections.Counter(row['kind'][7:].replace('-', '_') for row in bridge_events)
    for key in kinds:
        name = key.replace('-', '_')
        require(summary.get('n_' + name) == counts[name], 'summary event count disagrees: ' + key)
    require(counts['unsupported_kill'] >= 1 and counts['fall'] >= 1 and counts['link_broken'] == counts['repair_task'] == 1, 'declared first-collapse path not observed')
    require(summary.get('frames') == 120 and summary.get('done') is True and summary.get('first_collapse') is True, 'summary configuration/precondition mismatch')
    pending = sum(row['in_update'] for row in segments if row['scope'] == expected_scopes[-1])
    require(summary.get('state_valid') is True and summary.get('in_update_at_end') == pending and summary.get('landed_all') is (pending == 0), 'summary not derived from final actual state')
    singletons = ['cmb-hash','cmb-integrity','cmb-capability','cmb-damage','cmb-explosion','cmb-blast','cmb-install','cmb-repair','cmb-log']
    for scope in ['bridge:trace:before','bridge:trace:after','run']:
        for event in singletons:
            require(sum(row['event'] == event and row.get('scope') == scope for row in rows) == 1, 'missing/duplicate report ' + scope + ':' + event)
        timed = [row['kind'] for row in rows if row['event'] == 'cmb-timed' and row.get('scope') == scope]
        require(sorted(timed) == list(range(1, 7)), 'timed report cardinality')
    trust = [row for row in rows if row['event'] == 'cmb-integrity']
    require(len(trust) == 3 and all(row.get('trusted') == 1 and all(row.get(key) == 0 for key in ['depth_overflow','depth_unbalanced','identity_unknown','log_overflow','release_outside']) for row in trust), 'observer integrity failed')
    require(one('cmb-bridge-disarm').get('armed') == 0, 'observer was not disarmed')
    return {'status':'PASS','process_done':True,'trace':str(trace),'trace_bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'rows':len(rows),'boot':boot,'case':case,'summary':summary,'state_groups':len(heads),'segments_per_group':total,'contiguous_events':len(events),'bridge_events':dict(counts),'trusted_reports':len(trust),'before':heads[0],'immediate_after':heads[1],'final':heads[-1],'scope':'Original native bridge characterization only; not weapon-hit, alternate-host authority or fullpack acceptance'}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('trace', type=pathlib.Path)
    parser.add_argument('--run-json', type=pathlib.Path, required=True)
    parser.add_argument('--summary', type=pathlib.Path, required=True)
    args = parser.parse_args()
    try:
        result = audit(args.trace, args.run_json)
    except (ValueError, KeyError, TypeError, OSError) as error:
        result = {'status':'FAIL','reason':str(error),'trace':str(args.trace)}
    args.summary.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(result))
    sys.exit(0 if result['status'] == 'PASS' else 1)
