"""Slice 1a equivalence check. Run from the repository root:

    python -I eech-campaign/reference/m5-loss-structural/expand-check.py <base revision> [<candidate revision>]

The candidate is the working tree, or the named revision's files when one is given. The claim
is made for slice 1a's extraction commit only (docs/m5-loss-structural.md, "Revision boundary");
later reviewed semantic cuts change these files on purpose and are not token-equivalent.

1. gp_loss.c defines exactly the helpers in EXPANSIONS, and each helper's body,
   with its parameters renamed to the arguments of its call, is token for token
   the statements that call replaced.
2. In each of the six world destroy files, with every helper call expanded back
   into those statements, every `void name (entity *en)` function equals the
   base revision's token for token, and so does the rest of the file outside
   those functions.

Before comparing, both sides are normalised the same way:
- comments and blank space are removed;
- `#if DEBUG_MODULE ... #endif` blocks, including `#if DEBUG_MODULE >= 2`, are removed (each file must
  define DEBUG_MODULE 0, and no such block may hold an `#else` or `#elif`);
- `x->type` is read as `get_local_entity_type (x)` (the same expression for a non-null x);
- the local declarations at the top of each function are removed (the move deletes the ones
  its helpers absorbed; their types are not compared, which the build's own checks cover).
"""
import re
import subprocess
import sys

FILES = [
    'aphavoc/source/entity/mobile/aircraft/helicop/hc_dstry.c',
    'aphavoc/source/entity/mobile/aircraft/fixwing/fw_dstry.c',
    'aphavoc/source/entity/mobile/vehicle/routed/rv_dstry.c',
    'aphavoc/source/entity/mobile/vehicle/anti_air/aa_dstry.c',
    'aphavoc/source/entity/mobile/vehicle/person/ps_dstry.c',
    'aphavoc/source/entity/mobile/vehicle/ship/sh_dstry.c',
]

HELPERS = 'aphavoc/source/entity/special/group/gp_loss.c'

# each helper call as the world files make it, and the statements it replaced
EXPANSIONS = {
    'notify_member_task_dependents_completed (en);':
        'task = get_local_entity_first_child (en, LIST_TYPE_TASK_DEPENDENT); while (task) { destroy_task = task; '
        'task = get_local_entity_child_succ (task, LIST_TYPE_TASK_DEPENDENT); '
        'if (get_local_entity_type (destroy_task) == ENTITY_TYPE_TASK) { '
        'notify_local_entity (ENTITY_MESSAGE_TASK_COMPLETED, destroy_task, en, TASK_TERMINATED_OBJECTIVE_MESSAGE); } }',
    'increment_group_losses (group);':
        'losses = get_local_entity_int_value (group, INT_TYPE_LOSSES); losses ++; '
        'set_local_entity_int_value (group, INT_TYPE_LOSSES, losses);',
    'increment_group_primary_task_losses (group);':
        'task = get_local_group_primary_task (group); if (task) { '
        'losses = get_local_entity_int_value (task, INT_TYPE_LOSSES); losses ++; '
        'set_local_entity_int_value (task, INT_TYPE_LOSSES, losses); }',
    'notify_group_member_killed (group, en);': 'notify_local_entity (ENTITY_MESSAGE_MOBILE_KILLED, group, en);',
    'notify_forces_check_campaign_objectives (en);':
        'entity *force; force = get_local_entity_first_child (get_session_entity (), LIST_TYPE_FORCE); '
        'while (force) { notify_local_entity (ENTITY_MESSAGE_CHECK_CAMPAIGN_OBJECTIVES, force, en); '
        'force = get_local_entity_child_succ (force, LIST_TYPE_FORCE); }',
    'unlink_member_task_dependents (en);': 'unlink_local_entity_children (en, LIST_TYPE_TASK_DEPENDENT);',
    'unlink_member_from_guide (en);': 'delete_local_entity_from_parents_child_list (en, LIST_TYPE_FOLLOWER);',
    'unlink_member_from_group (en);': 'delete_local_entity_from_parents_child_list (en, LIST_TYPE_MEMBER);',
    'unlink_member_from_takeoff_queue (en);': 'delete_local_entity_from_parents_child_list (en, LIST_TYPE_TAKEOFF_QUEUE);',
}

C_OPERATORS = r'->|\+\+|--|<<=|>>=|<<|>>|<=|>=|==|!=|&&|\|\||[-+*/%&|^]='
TOKEN = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|[A-Za-z_]\w*|\d+\.?\d*|' + C_OPERATORS + r'|\S')

FUNCTION = r'(?:static )?void (\w+) \(entity \*en\)'


def normalise_source(text):
    text = re.sub(r'/\*.*?\*/', ' ', text, flags=re.S)
    text = re.sub(r'//[^\n]*', ' ', text)
    for block in re.findall(r'#if DEBUG_MODULE[^\n]*\n.*?#endif', text, flags=re.S):
        if re.search(r'#el(?:se|if)', block):
            raise SystemExit(f'a DEBUG_MODULE block has an alternative branch: {block[:60]!r}')
    text = re.sub(r'#if DEBUG_MODULE[^\n]*\n.*?#endif', ' ', text, flags=re.S)
    return re.sub(r'(\w+)->type\b', r'get_local_entity_type (\1)', text)


def split_definitions(text, signature):
    """(name -> body text, the text outside those bodies), for each file-level definition matching signature"""
    bodies, outside, last = {}, [], 0
    for m in re.finditer(r'^' + signature + r'\n\{', text, re.M):
        depth, i = 1, m.end()
        while depth:
            depth += {'{': 1, '}': -1}.get(text[i], 0)
            i += 1
        bodies[m.group(1)] = text[m.end():i - 1]
        outside.append(text[last:m.end()])
        last = i - 1
    outside.append(text[last:])
    return bodies, ' '.join(outside)


def without_leading_declarations(tokens):
    body = ' '.join(tokens)
    return re.sub(r'^(?:(?!return )\w+ (?:\* )?\w+(?: , (?:\* )?\w+)* ; )+', '', body)


def read_candidate(path, candidate):
    if candidate is None:
        return open(path, encoding='latin-1').read()
    return subprocess.run(['git', 'show', f'{candidate}:{path}'], capture_output=True, check=True).stdout.decode('latin-1')


def helpers_match_expansions(candidate=None):
    m = re.compile(r'void (\w+) \(([^)]*)\)')
    source = read_candidate(HELPERS, candidate)
    if re.search(r'^\s*#\s*if', source, re.M):
        print('FAIL gp_loss.c has conditional compilation, which the comparison would not see')
        return 1
    text = normalise_source(source)
    bodies, _ = split_definitions(text, r'(?:static )?\w+ (\w+) \([^)]*\)')
    parameters = {name: [p.split('*')[-1].strip() for p in params.split(',')] for name, params in m.findall(text)}
    calls = {call.split(' ')[0]: call for call in EXPANSIONS}
    failures = 0
    if set(bodies) != set(calls):
        print(f'FAIL gp_loss.c helpers {sorted(set(bodies) ^ set(calls))}')
        failures += 1
    for name in sorted(set(bodies) & set(calls)):
        arguments = [a.strip() for a in re.search(r'\((.*)\)', calls[name]).group(1).split(',')]
        renamed = dict(zip(parameters[name], arguments))
        body = re.sub(r'\b\w+\b', lambda w: renamed.get(w.group(0), w.group(0)), bodies[name])
        same = without_leading_declarations(TOKEN.findall(body)) == without_leading_declarations(TOKEN.findall(EXPANSIONS[calls[name]]))
        same = same and len(parameters[name]) == len(arguments)
        print(f"{'SAME' if same else 'DIFF'} gp_loss.c {name}")
        failures += not same
    return failures


def file_matches_base(path, base, candidate=None):
    current = read_candidate(path, candidate)
    if not re.search(r'^#define DEBUG_MODULE 0$', current, re.M):
        print(f'FAIL {path}: DEBUG_MODULE is not 0, so removing its blocks is not equivalent')
        return 1
    original = subprocess.run(['git', 'show', f'{base}:{path}'], capture_output=True, check=True).stdout.decode('latin-1')
    expanded = normalise_source(current)
    for call, statements in EXPANSIONS.items():
        expanded = expanded.replace(call, statements)
    old, old_outside = split_definitions(normalise_source(original), FUNCTION)
    new, new_outside = split_definitions(expanded, FUNCTION)
    name = path.split('/')[-1]
    failures = 0
    if old.keys() != new.keys() or TOKEN.findall(old_outside) != TOKEN.findall(new_outside):
        print(f'DIFF {name} outside its functions, or its function list')
        failures += 1
    for function in [f for f in old if f in new]:
        same = without_leading_declarations(TOKEN.findall(old[function])) == without_leading_declarations(TOKEN.findall(new[function]))
        print(f"{'SAME' if same else 'DIFF'} {name} {function}")
        failures += not same
    return failures


def main():
    base = sys.argv[1]
    candidate = sys.argv[2] if len(sys.argv) > 2 else None
    failures = helpers_match_expansions(candidate) + sum(file_matches_base(path, base, candidate) for path in FILES)
    print('PASS' if failures == 0 else f'FAIL: {failures}')
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
