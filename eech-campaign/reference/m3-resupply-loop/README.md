# M3: one campaign → world → campaign resupply loop

The retained evidence of the M3 resupply-loop slice. What it shows and what it
does not: [`docs/m3-resupply-loop.md`](../../docs/m3-resupply-loop.md).

- `observations.jsonl.gz`: the reference run's structured observations with the supply detail (`observe_supply=1`)
- `chain-jester.jsonl`: the featured chain's own events (group 88235 and the Halat and Beirut supply changes)
- `chains.json`: `tools/supply-chain-check.py` on these observations: every delivery and chain
- `check.json`: `tools/observation-check.py` on these observations against the recording
- `summary.txt`, `BUILD-INFO.txt`: the runner's summary; the binaries' and scripts' hashes

The run's Tacview recording and metrics are byte-identical to M2's
[`../lebanon-3h/`](../lebanon-3h/) files.

```sh
python ../../tools/supply-chain-check.py observations.jsonl.gz ../lebanon-3h/metrics.json
python ../../tools/observation-check.py ../lebanon-3h/recording.zip.acmi observations.jsonl.gz ../lebanon-3h/metrics.json
```
