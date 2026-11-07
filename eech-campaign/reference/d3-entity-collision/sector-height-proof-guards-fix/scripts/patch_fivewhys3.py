import sys

P = sys.argv[1] + "/docs/governance/D3-FIVE-WHYS-20261102.md"
raw = open(P, "rb").read()
assert b"\r\n" not in raw
d = raw.decode("utf-8")
anchor = "\nSuccess is the next reviewed production increment with unchanged accepted runs.\n"
assert d.count(anchor) == 1
new = (
    "- An expected error string is only evidence when the run reached the actual eligible intervention it describes: credit a refusal only with exactly that one intervention (height_altered=1), whatever the\n"
    "  secondary input altered and whatever marker the engine stopped at; any fatal before it is a failure, even one bearing the expected marker, and only a clean run with no eligible read is UNREACHED. A\n"
    "  matching marker string does not prove the cause.\n"
    "- Scoped whole data needs unique, complete rows and a terminal verdict on BOTH sides of a comparison: a green prefix is not a whole scan. Validate the accepted report and the live one as whole (verdict last, declared\n"
    "  count, unique contiguous indices and sector keys, totals equal to the rows, exact fields and domains) before comparing, keep the hosted scope narrower than the original's, and run single-fault controls from a\n"
    "  coherent positive; check the real retained report against the same rules, not only scripted fixtures.\n"
)
d = d.replace(anchor, new + anchor)
open(P, "wb").write(d.encode("utf-8"))
print("ok")
