import json
import sys

pre = json.load(open(sys.argv[1], encoding="utf-8"))
post = json.load(open(sys.argv[2], encoding="utf-8"))
extra_pre = json.load(open(sys.argv[3], encoding="utf-8"))
extra_post = json.load(open(sys.argv[4], encoding="utf-8"))
out = sys.argv[5]
files_pre = pre["files"]
files_post = post["files"]
only_pre = sorted(set(files_pre) - set(files_post))
only_post = sorted(set(files_post) - set(files_pre))
differing = sorted(p for p in files_pre if p in files_post and files_pre[p] != files_post[p])
disagree_key = "files_whose_raw_bytes_and_git_filtered_identity_disagree_with_the_committed_blob"
extra_equal = extra_pre["files"] == extra_post["files"]
result = {
    "pre_fingerprint": pre["fingerprint_of_raw_sha256_map"], "post_fingerprint": post["fingerprint_of_raw_sha256_map"],
    "file_count_pre": pre["file_count"], "file_count_post": post["file_count"],
    "rows_only_in_pre": only_pre, "rows_only_in_post": only_post, "rows_that_differ": differing,
    "all_rows_equal": not (only_pre or only_post or differing) and pre["fingerprint_of_raw_sha256_map"] == post["fingerprint_of_raw_sha256_map"],
    "head_sha": pre["head_sha"], "changed_from_head_count": [pre["changed_from_head_count"], post["changed_from_head_count"]],
    "deleted_from_head": [pre["deleted_from_head"], post["deleted_from_head"]],
    "raw_vs_git_filtered_disagreements": {"pre": pre[disagree_key], "post": post[disagree_key]},
    "extra_identity_equal": extra_equal, "extra_identity_aggregate": [extra_pre["aggregate_sha256"], extra_post["aggregate_sha256"]],
}
json.dump(result, open(out, "w", encoding="utf-8", newline="\n"), indent=2)
print(json.dumps({k: v for k, v in result.items() if k not in ("raw_vs_git_filtered_disagreements", "rows_that_differ")}, indent=1))
print("disagreements:", len(pre[disagree_key]), [d if isinstance(d, str) else d.get("path") for d in pre[disagree_key]])
