# Reference scenario: retail Lebanon, 3 simulated hours

The retained evidence of the M2 reference scenario. What it is, how it was
produced and checked, and what it does and does not establish:
[`docs/m2-reference.md`](../../docs/m2-reference.md).

- `recording.zip.acmi`: the Tacview recording (Tacview opens it directly)
- `observations.jsonl.gz`: the structured observations of the same samples
- `metrics.json`: the run's metrics (equal to `regression/windows/lebanon_retail.json`)
- `inputs.json`: SHA-256 of every input file (retail data is not in the repository)
- `check.json`: `tools/observation-check.py` on the three files above
- `summary.txt`, `BUILD-INFO.txt`: the runner's summary; the binaries' and scripts' hashes

```sh
python ../../tools/observation-check.py recording.zip.acmi observations.jsonl.gz metrics.json
```
