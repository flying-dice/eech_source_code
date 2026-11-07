p = "docs/governance/D3-FIVE-WHYS-20261102.md"
d = open(p, "rb").read().decode("utf-8")
crlf = b"\r\n" in open(p, "rb").read()
assert not crlf
old = """
Success is the next reviewed production increment with unchanged accepted runs.
"""
new = """- When a raw word moves to the host, every classifier of that word is part of the move: a check that demands
  every fact be finite (the integer classifier and its floating mirror alike) silently turns a valid raw
  domain (a restored negative, infinite or NaN word) into a refusal. Audit and pin both, with the exact
  words, before the first question carries the word. Detect an arrival by the key the insertion issued,
  not by a record's first statement: records merge, keys do not.
- A move changes the entry point of an accepted control, not its scenarios: keep the original body compiled
  in the profile that keeps the original solver, run the accepted expectations there unchanged, and add the
  differential of the host's expression beside it; a retired field is never read or written in the hosted
  build (pinned by a census guard), instead of being kept as a mirror that can go stale.

Success is the next reviewed production increment with unchanged accepted runs.
"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))

p = "eech-campaign/docs/d3-sector-height-contract.md"
d = open(p, "rb").read().decode("utf-8")
old = "## 6. The concrete next production move (after the long-run blocker closes; NOT built here)\n"
new = """## 6. The concrete next production move (PROPOSED design; SUPERSEDED by the production contract)

> **Update (7 November 2026, Task49).** This section was a proposal. The production move is built in source and its contract, corrected from the actual source, is
> [d3-sector-height-production-contract.md](d3-sector-height-production-contract.md) (arrival detected by the key, no mirror, the getter as the one read, fact 12 as the host's word, the city's shape fact,
> phases 17 and 18). Sections 1 to 4 above remain the characterization of the ORIGINAL; the original-solver profile (`collision-fixture`) keeps that body and the 179-expectation control runs there unchanged.
> The text below is kept as written for the record.

"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
