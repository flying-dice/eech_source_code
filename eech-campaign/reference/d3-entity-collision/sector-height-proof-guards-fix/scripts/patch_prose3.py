import sys

ROOT = sys.argv[1] + "/eech-campaign/"


def patch(rel, pairs):
    p = ROOT + rel
    raw = open(p, "rb").read()
    assert b"\r\n" not in raw, rel
    d = raw.decode("utf-8")
    for old, new in pairs:
        assert d.count(old) == 1, (rel, old[:90], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))


patch("crates/eech-world/lua/sector-height-baseline.lua", [(
"""-- `accepted`, a scan that is whole with at least one read) ends the process non-zero after the observation is closed; nothing is searched, extended or retried to reach a green.
""", """-- `accepted`, a scan that is whole with at least one read) ends the process non-zero after the observation is closed; nothing is searched, extended or retried to reach a green.
-- BOTH SIDES ARE VALIDATED AS A WHOLE SCOPED SCAN BEFORE ANYTHING IS COMPARED (ENV-SECTOR-HEIGHT-SCOPED-REPORT-COMPLETE-1): the accepted ORIGINAL report must hold exactly one identity line, one `sho-scan` summary, one
-- `sho-tables` line and one `sho-verdict whole=1 events= clearances= sectors=` line, that verdict LAST and its sector count the rows'; both sides need the exact scoped fields and domains, the declared sector count, unique
-- contiguous row indices from 0, unique sector keys, `firsts` equal to `reads`, no fact-12 offer before a FIRST, `scan_out_of_range=0`, the rows' reads and fact-12 counts summing to the totals and each row's words
-- consistent with its changes. Never required of the hosted side: the original-link `sho-complete value=1`, the events, the raise or the session. A malformed or incomplete accepted report is INVALID_ACCEPTED (an
-- unreadable one NOT_COMPARABLE); a malformed or incomplete live hosted scan is INVALID_HOSTED and `scan_whole=0`; UNREACHED only for a clean, whole scan with no read. Duplicates are refused, never collapsed.
""")])

patch("crates/eech-world/lua/entity-authority.lua", [(
"""-- No actor, ray, duration or seed is searched: a window with no eligible read is UNREACHED. After a fatal stop nothing is read from the engine.
""", """-- No actor, ray, duration or seed is searched: a window with no eligible read is UNREACHED. After a fatal stop nothing is read from the engine.
-- A refusal mode is credited ONLY with EXACTLY the one actual eligible intervention (height_altered=1), independent of every secondary-input alteration count (the configured forced hit) and of marker equality: ANY fatal
-- before that intervention is a FAILURE, the mode's own marker included (an unrelated fatal bearing it is not a cause), and only a clean run with no eligible read is UNREACHED (ENV-SECTOR-HEIGHT-REFUSAL-INTERVENTION-1).
""")])

patch("docs/d3-sector-height-production-contract.md", [
("""(the accepted
evidence reused at its true scope: 63 FIRST reads over 2 sectors and 27615 fact-12 offers, each equal to its FIRST). **Not compared:**""", """(the accepted
evidence reused at its true scope: 63 FIRST reads over 2 sectors and 27615 fact-12 offers, each equal to its FIRST). **Both sides are validated as a WHOLE scoped scan before anything is compared:** the accepted original must carry
exactly one identity, one `sho-scan` summary, one `sho-tables` and one terminal `sho-verdict whole=1 ... sectors=N` (the real f01 report does: 2 sectors, 63 reads, 27615 offers, checked by an independent port of the same rules); both
sides need the exact scoped fields and domains, unique contiguous row indices, unique sector keys, the declared sector count and totals and words consistent with the rows; the hosted side is NOT asked for the original-link
`sho-complete=1`, events, raise or session. Damaged input is `INVALID_ACCEPTED` or `INVALID_HOSTED` (never EQUAL_AT_SCOPE, SCAN_ONLY or `scan_whole=1`). **Not compared:**"""),
("""`NOT_COMPARABLE` (another window or an unreadable report) and `UNREACHED` (no scan FIRST) are not passes.""", """`NOT_COMPARABLE` (another window or an unreadable report), `INVALID_ACCEPTED`, `INVALID_HOSTED` and `UNREACHED` (a clean, whole scan with no FIRST) are not passes."""),
("""the changed-input and refusal modes each alter ONE eligible read and credit the NAMED outcome:""", """the changed-input and refusal modes each alter ONE eligible read and credit the NAMED outcome, **only with exactly that one intervention (`height_altered=1`), independent of the configured secondary hit and of marker
equality: any fatal before it is FAILURE (the mode's own marker included) and only a clean run with no eligible read is UNREACHED**:"""),
])
print("ok")
