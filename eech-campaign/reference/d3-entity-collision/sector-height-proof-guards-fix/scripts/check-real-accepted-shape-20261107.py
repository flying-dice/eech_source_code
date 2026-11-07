"""Reads the retained ACCEPTED original report (private, outside Git) and checks it against the SAME rules as lua/sector-height-baseline.lua's validate_scan/load_accepted, with an independent Python port. Prints safe aggregates only."""
import re
import sys

path = sys.argv[1]
ident, totals, tables, rows, verdicts, last = [], [], [], [], [], None


def strict(line):
    out, first = {}, True
    for token in line.split():
        if first:
            first = False
            continue
        m = re.match(r"^([\w]+)=(.+)$", token)
        if not m or m.group(1) in out:
            raise SystemExit("malformed token in " + line[:30])
        out[m.group(1)] = m.group(2)
    return out


with open(path, "rb") as f:
    for raw in f:
        line = raw.decode("utf-8", "replace").rstrip("\r\n")
        if line:
            last = line
        if line.startswith("sho-baseline "):
            ident.append(strict(line))
        elif line.startswith("sho-scan reads="):
            totals.append(strict(line))
        elif line.startswith("sho-scan i="):
            rows.append(strict(line))
        elif line.startswith("sho-tables "):
            tables.append(strict(line))
        elif line.startswith("sho-verdict "):
            verdicts.append(line)
print("counts identity/totals/tables/verdicts/rows:", len(ident), len(totals), len(tables), len(verdicts), len(rows))
assert (len(ident), len(totals), len(tables), len(verdicts)) == (1, 1, 1, 1)
m = re.match(r"^sho-verdict whole=1 events=(\d+) clearances=(\d+) sectors=(\d+)$", verdicts[0])
assert m and last == verdicts[0], "no complete terminal verdict"
whole = lambda s: int(s) if re.match(r"^\d{1,15}$", s or "") else None
T = {k: whole(totals[0].get(k)) for k in ("reads", "firsts", "sectors", "fact_offers", "fact_nonzero", "fact_equal_first", "fact_differ_first", "fact_without_first")}
assert all(v is not None for v in T.values()), T
assert whole(tables[0].get("scan_out_of_range")) == 0
assert T["firsts"] == T["reads"] and T["fact_without_first"] == 0 and T["fact_nonzero"] <= T["fact_offers"] and T["fact_offers"] == T["fact_equal_first"] + T["fact_differ_first"]
assert len(rows) == T["sectors"] == int(m.group(3))
seen, s = set(), dict(reads=0, off=0, eq=0, df=0)
for pos, r in enumerate(rows):
    n = {k: whole(r.get(k)) for k in ("i", "sx", "sz", "reads", "first_seq", "last_seq", "changes", "fact_offers", "fact_equal_first", "fact_differ_first", "fact_changes")}
    assert all(v is not None for v in n.values()), r
    assert all(re.match(r"^0x[0-9a-f]{8}$", r.get(k, "")) for k in ("first_word", "last_word", "fact_first_word", "fact_last_word")), r
    assert n["i"] == pos and (r["sx"], r["sz"]) not in seen
    seen.add((r["sx"], r["sz"]))
    assert n["reads"] >= 1 and n["first_seq"] <= n["last_seq"] and n["changes"] <= n["reads"] - 1
    assert n["changes"] > 0 or r["first_word"] == r["last_word"]
    assert n["fact_equal_first"] + n["fact_differ_first"] == n["fact_offers"]
    assert n["fact_offers"] > 0 or (n["fact_changes"] == 0 and r["fact_first_word"] == r["fact_last_word"] == "0x00000000")
    assert n["fact_offers"] == 0 or (n["fact_changes"] <= n["fact_offers"] - 1 and (n["fact_changes"] > 0 or r["fact_first_word"] == r["fact_last_word"]))
    s["reads"] += n["reads"]; s["off"] += n["fact_offers"]; s["eq"] += n["fact_equal_first"]; s["df"] += n["fact_differ_first"]
assert s["reads"] == T["reads"] and s["off"] == T["fact_offers"] and s["eq"] == T["fact_equal_first"] and s["df"] == T["fact_differ_first"]
print("VALID by the same rules: sectors", T["sectors"], "reads", T["reads"], "fact_offers", T["fact_offers"], "identity window", {k: ident[0].get(k) for k in ("scenario", "seed", "frame_ms", "hours", "frames")})
