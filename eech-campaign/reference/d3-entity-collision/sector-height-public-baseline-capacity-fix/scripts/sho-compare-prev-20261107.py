"""Exact changed-input join between the d74 final POST map and the join-fix final POST map, and the tool/script identity rows that did not change (reuse by exact join, not by claim)."""
import json
import sys

prev = json.load(open(sys.argv[1], encoding="utf-8"))
cur = json.load(open(sys.argv[2], encoding="utf-8"))
prev_extra = json.load(open(sys.argv[3], encoding="utf-8"))
cur_extra = json.load(open(sys.argv[4], encoding="utf-8"))
out = sys.argv[5]
fp, fc = prev["files"], cur["files"]
changed = sorted(p for p in fp if p in fc and fp[p]["raw_sha256"] != fc[p]["raw_sha256"])
added = sorted(set(fc) - set(fp))
removed = sorted(set(fp) - set(fc))
pe = {r["path"]: r["sha256"] for r in prev_extra["files"]}
ce = {r["path"]: r["sha256"] for r in cur_extra["files"]}
extra_changed = sorted(p for p in pe if p in ce and pe[p] != ce[p])
extra_same = sorted(p for p in pe if p in ce and pe[p] == ce[p])
result = {
    "previous_post_fingerprint": prev["fingerprint_of_raw_sha256_map"], "current_post_fingerprint": cur["fingerprint_of_raw_sha256_map"],
    "files_previous": prev["file_count"], "files_current": cur["file_count"],
    "map_rows_changed_since_the_d74_final_post": changed, "map_rows_added": added, "map_rows_removed": removed,
    "unchanged_map_rows": len(fp) - len(changed) - len(removed),
    "tool_and_script_rows_changed": extra_changed, "tool_and_script_rows_unchanged": extra_same,
}
json.dump(result, open(out, "w", encoding="utf-8", newline="\n"), indent=2)
print(json.dumps({k: v for k, v in result.items() if k not in ("tool_and_script_rows_unchanged",)}, indent=1))
print("unchanged tool rows:", extra_same)
