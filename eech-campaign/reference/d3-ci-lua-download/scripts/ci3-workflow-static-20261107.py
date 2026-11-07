"""Static check of the workflow change against the exact 1b89 workflow (no YAML parser or actionlint exists locally; the CI parse is Root's rerun).
The only differences from 1b89 must be: the Lua step's curl+sha lines replaced by the pinned-variable + helper call (two URLs, lua.org first) + the same sha check, and the one added selftest step in evidence-selftest.
Everything else, the version, the SHA-256, the compiler and link flags and the matrix, is byte-identical. Usage: check.py <repo>"""
import difflib
import json
import re
import subprocess
import sys

repo = sys.argv[1]
OLD_REV = "1b89c0b0ab613d2f6a08ea02dd6612457efa8982"
PIN = "2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333"
path = ".github/workflows/eech-campaign.yml"
old = subprocess.run(["git", "-C", repo, "show", OLD_REV + ":" + path], capture_output=True, check=True).stdout.decode("utf-8").splitlines()
new = open(repo + "/" + path, "rb").read().decode("utf-8").splitlines()
problems = []
removed, added = [], []
for tag, a0, a1, b0, b1 in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes():
    if tag != "equal":
        removed.append(old[a0:a1])
        added.append(new[b0:b1])
# hunks are in file order (the evidence-selftest job precedes windows-cross): classify by content, not by position
expected_removed = ['          curl -fsSL -o lua.tar.gz https://www.lua.org/ftp/lua-5.1.5.tar.gz', '          echo "' + PIN + '  lua.tar.gz" | sha256sum -c -']
if sorted(removed, key=len) != [[], expected_removed] or len(added) != 2:
    problems.append("removed hunks are not exactly the curl and the sha check lines: %r" % removed)
pairs = sorted(zip(removed, added), key=lambda pair: len(pair[0]))
lua_block = pairs[1][1] if len(pairs) == 2 else []
text = "\n".join(lua_block)
if [pairs[0][1]] != [[
        "      - name: Pinned-archive downloader",
        "        # the Lua archive downloader of windows-cross, against a fake transport (no network)",
        "        run: sh tools/fetch-pinned-archive-selftest.sh"]]:
    problems.append("the added selftest step is not exactly as stated: %r" % [pairs[0][1]])
for need in ("lua_sha256=" + PIN, 'sh "$GITHUB_WORKSPACE/eech-campaign/tools/fetch-pinned-archive.sh" lua.tar.gz "$lua_sha256"',
             "https://www.lua.org/ftp/lua-5.1.5.tar.gz", "https://distfiles.macports.org/lua/lua-5.1.5.tar.gz", 'echo "$lua_sha256  lua.tar.gz" | sha256sum -c -'):
    if need not in text:
        problems.append("missing in the Lua step: " + need)
if text.find("www.lua.org") > text.find("distfiles.macports.org"):
    problems.append("the official URL is not first")
if re.search(r"(^|\s)curl(\s|$)", "\n".join(l for l in lua_block if not l.lstrip().startswith("#"))):
    problems.append("a bare curl remains in the Lua step")
if len(re.findall(PIN, "\n".join(new))) != 1 or len(re.findall(PIN, "\n".join(old))) != 1:
    problems.append("the pin must appear once (the variable the helper and the check both use), and once before: now %d, before %d" % (len(re.findall(PIN, "\n".join(new))), len(re.findall(PIN, "\n".join(old)))))
for line in lua_block:
    if line.strip() and not line.startswith(" " * 10):
        problems.append("the Lua step line is not at the run-block indentation: %r" % line)
for flag in ("-O2 -DLUA_BUILD_AS_DLL -c", "-shared -o ../../lua-mingw/lua.dll *.o -Wl,--out-implib,../../lua-mingw/liblua.dll.a"):
    if flag not in "\n".join(new) or flag not in "\n".join(old):
        problems.append("compiler/link flags missing: " + flag)
dockerfile_same = subprocess.run(["git", "-C", repo, "diff", "--quiet", OLD_REV, "--", "eech-campaign/tools/Dockerfile"]).returncode == 0
if not dockerfile_same:
    problems.append("tools/Dockerfile differs from 1b89")
print(json.dumps({"removed_lines": sum(len(x) for x in removed), "added_lines": sum(len(x) for x in added), "dockerfile_unchanged": dockerfile_same, "problems": problems}, indent=1))
sys.exit(1 if problems else 0)
