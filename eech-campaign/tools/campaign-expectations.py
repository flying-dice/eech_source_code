#!/usr/bin/env python3
"""Checks that a campaign run played out as an EECH dynamic campaign should,
from its metrics file (campaign.lua metrics=...; lua/metrics.lua), without a
reference run: the mechanics each regression scenario must show in its three
simulated hours.

The expectations are the mechanics every correct run of the scenario showed,
across seeds and platforms: which task types get flown, and that combat,
attrition, reinforcement, supply, production and (Georgia) captures all
happen. A run can differ from the baseline event by event and still pass
here; a broken mechanic fails.

Usage: campaign-expectations.py <metrics.json> [<scenario>]
Exit status: 0 when every expectation holds.
"""
import json, sys

COMMON_AIR = {'BAI', 'BDA', 'CLOSE_AIR_SUPPORT', 'COMBAT_AIR_PATROL', 'ESCORT', 'GROUND_STRIKE', 'OCA_SWEEP',
              'RECON', 'REPAIR', 'SEAD', 'SUPPLY', 'TRANSFER_FIXED_WING', 'TRANSFER_HELICOPTER'}
EXPECTED = {
    # the complete retail economy (factories, refineries, ports), regen every 600 s
    'lebanon_retail': {
        'air tasks': COMMON_AIR | {'OCA_STRIKE'},
        'ground tasks': {'ADVANCE', 'RETREAT', 'TROOP_MOVEMENT_PATROL'},
        'regen': True, 'production': True, 'captures': False, 'resupply': True,
    },
    # the retail map3 campaign: regen every 16.7 h (none in 3 h), no producers, FARPs change hands
    'georgia_retail': {
        'air tasks': COMMON_AIR | {'TROOP_INSERTION'},
        'ground tasks': {'ADVANCE', 'RETREAT', 'TROOP_MOVEMENT_INSERT_CAPTURE', 'TROOP_MOVEMENT_PATROL'},
        'regen': False, 'production': False, 'captures': True, 'resupply': False,
    },
}


def check(metrics, scenario):
    e = EXPECTED[scenario]
    first, last = metrics['checkpoints'][0], metrics['checkpoints'][-1]
    c = last['counts']
    results = []

    def expect(name, ok, detail):
        results.append((ok, name, detail))

    air, ground = set(), set()
    for key, n in c['sorties'].items():
        side, kind, task = key.split(' ', 2)
        if n > 0:
            (air if kind in ('helicopter', 'fixed_wing') else ground).add(task)
    for task in sorted(e['air tasks']):
        expect(f'air task {task} flown', task in air, '')
    for task in sorted(e['ground tasks']):
        expect(f'ground task {task} ordered', task in ground, '')
    for side in ('blue', 'red'):
        launched = sum(n for k, n in c['launched'].items() if k.startswith(side + ' '))
        expect(f'{side} fights: weapons launched', launched > 0, launched)
        for kind in ('helicopter', 'ground_vehicle'):
            lost = c['lost'].get(f'{side} {kind}', 0)
            expect(f'{side} {kind} losses', lost > 0, lost)
        supply = sum(n for k, n in c['sorties'].items() if k.startswith(side + ' ') and k.endswith(' SUPPLY'))
        if side == 'red' or e['production']:
            expect(f'{side} flies SUPPLY', supply > 0, supply)
        for res in ('ammo', 'fuel'):
            key = f'{side} KEYSITE_AIRBASE {res}'
            used = first['supply'].get(key, 0) - last['supply'].get(key, 0)
            expect(f'{side} airbases consume {res}', used > 0, round(used, 1))
        if e['regen']:
            for kind in ('fixed_wing', 'helicopter'):
                n = c['spawned'].get(f'{side} {kind}', 0)
                expect(f'{side} {kind} regenerated from reserves', n > 0, n)
        if e['production']:
            for kind in ('KEYSITE_FACTORY', 'KEYSITE_OIL_REFINERY'):
                n = last['keysite_states'].get(f'{side} {kind} usable', 0)
                expect(f'{side} {kind[8:].lower()} producing', n > 0, n)
    if e['resupply']:
        # SUPPLY deliveries reach airbases: ammo from factories, fuel from refineries and airbases
        for res in ('ammo', 'fuel'):
            n = sum(v for k, v in c.get('resupplied', {}).items() if k.endswith(f' KEYSITE_AIRBASE {res}'))
            expect(f'airbases resupplied with {res}', n > 0, n)
    if e['captures']:
        n = sum(c['captures'].values())
        expect('keysites change hands', n > 0, n)
    return results


def main():
    if len(sys.argv) not in (2, 3):
        print(__doc__)
        return 2
    metrics = json.load(open(sys.argv[1]))
    scenario = sys.argv[2] if len(sys.argv) == 3 else metrics['run']['scenario']
    results = check(metrics, scenario)
    failed = [r for r in results if not r[0]]
    for ok, name, detail in results:
        if not ok:
            print(f'  EXPECTATION FAILED  {name} ({detail})')
    print(f'{scenario}: {len(results) - len(failed)} of {len(results)} campaign expectations hold')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main())
