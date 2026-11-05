#!/usr/bin/env python3
# Builds the probe mission: a copy of an installed DCS mission (the template)
# with every unit and module requirement removed and one MISSION START trigger
# whose DO SCRIPT sets EECH_DCS_PROBE and runs eech-dcs-probe.lua. The mission
# file is a Lua table; this parses the subset DCS writes and serialises it
# back with sorted keys, so the output is deterministic for a given template.
#
#   python -I build-probe-miz.py <template.miz> <out.miz> <stage> <probe.lua> <dll_dir> <root> <log_file>
import re
import sys
import zipfile


class Parser:
    def __init__(self, text):
        self.s, self.i = text, 0

    def skip(self):
        while True:
            m = re.compile(r"\s+|--[^\n]*").match(self.s, self.i)
            if not m or m.end() == self.i:
                return
            self.i = m.end()

    def value(self):
        self.skip()
        c = self.s[self.i]
        if c == "{":
            return self.table()
        if c == '"':
            return self.string()
        m = re.compile(r"-?\d+(\.\d+)?([eE][-+]?\d+)?|true|false|nil").match(self.s, self.i)
        if not m:
            raise ValueError(f"unexpected {self.s[self.i:self.i + 40]!r}")
        self.i = m.end()
        t = m.group()
        if t in ("true", "false", "nil"):
            return {"true": True, "false": False, "nil": None}[t]
        return float(t) if any(x in t for x in ".eE") else int(t)

    def string(self):
        self.i += 1
        out = []
        escapes = {"n": "\n", "t": "\t", "r": "\r", '"': '"', "\\": "\\", "'": "'", "\n": "\n", "a": "\a", "b": "\b", "f": "\f", "v": "\v"}
        while True:
            c = self.s[self.i]
            if c == '"':
                self.i += 1
                return "".join(out)
            if c == "\\":
                n = self.s[self.i + 1]
                if n.isdigit():
                    m = re.compile(r"\d{1,3}").match(self.s, self.i + 1)
                    out.append(chr(int(m.group())))
                    self.i = m.end()
                    continue
                out.append(escapes[n])
                self.i += 2
                continue
            out.append(c)
            self.i += 1

    def table(self):
        self.i += 1
        t = {}
        while True:
            self.skip()
            if self.s[self.i] == "}":
                self.i += 1
                return t
            if self.s[self.i] != "[":
                raise ValueError(f"expected [key] at {self.s[self.i:self.i + 40]!r}")
            self.i += 1
            key = self.value()
            self.skip()
            assert self.s[self.i] == "]", self.s[self.i:self.i + 20]
            self.i += 1
            self.skip()
            assert self.s[self.i] == "=", self.s[self.i:self.i + 20]
            self.i += 1
            t[key] = self.value()
            self.skip()
            if self.s[self.i] == ",":
                self.i += 1


def parse_chunk(text, name):
    m = re.match(r"\s*" + name + r"\s*=\s*", text)
    if not m:
        raise ValueError(f"no '{name} =' chunk")
    p = Parser(text)
    p.i = m.end()
    return p.value()


def lua_string(s):
    out = ['"']
    for ch in s:
        if ch in '"\\':
            out.append("\\" + ch)
        elif ch == "\n":
            out.append("\\n")
        elif ord(ch) < 32 or ord(ch) == 127:
            out.append(f"\\{ord(ch)}")
        else:
            out.append(ch)
    out.append('"')
    return "".join(out)


def lua_value(v, depth=1):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float)):
        return repr(v)
    if isinstance(v, str):
        return lua_string(v)
    pad = "    " * depth
    keys = sorted(v, key=lambda k: (isinstance(k, str), k))
    body = "".join(f"{pad}[{lua_value(k)}] = {lua_value(v[k], depth + 1)},\n" for k in keys)
    return "{\n" + body + "    " * (depth - 1) + "}"


def main(argv):
    if len(argv) != 8:
        print("usage: build-probe-miz.py <template.miz> <out.miz> <stage> <probe.lua> <dll_dir> <root> <log_file>", file=sys.stderr)
        return 2
    template, out, stage, probe, dll_dir, root, log_file = argv[1:]
    with zipfile.ZipFile(template) as z:
        mission = parse_chunk(z.read("mission").decode("utf-8"), "mission")
        others = {n: z.read(n) for n in z.namelist() if n != "mission"}
    for name in (probe, dll_dir, root, log_file):
        if "]]" in name:
            raise ValueError(f"path contains ]]: {name}")
    script = (f'EECH_DCS_PROBE = {{ stage = "{stage}", dll_dir = [[{dll_dir}]], root = [[{root}]], log_file = [[{log_file}]] }}\n'
              f"dofile ([[{probe}]])")
    mission["needModules"] = {}
    mission["usedModules"] = {}
    for side in mission.get("coalition", {}).values():
        for country in side.get("country", {}).values():
            for kind in ("plane", "helicopter", "vehicle", "ship", "static"):
                country.pop(kind, None)
    mission["trig"] = {
        "actions": {1: f"a_do_script({lua_string(script)});"}, "events": {}, "custom": {}, "func": {},
        "flag": {1: True}, "conditions": {1: "return(true)"}, "customStartup": {},
        "funcStartup": {1: "if mission.trig.conditions[1]() then mission.trig.actions[1]() end"},
    }
    mission["trigrules"] = {1: {"rules": {}, "eventlist": "", "predicate": "triggerStart", "comment": "EECH DCS probe",
                                "actions": {1: {"text": script, "predicate": "a_do_script"}}}}
    mission["descriptionText"] = "EECH DCS probe (stage " + stage + "): no units; one MISSION START DO SCRIPT."
    # fixed entry timestamps: the same inputs give a byte-identical .miz
    def entry(name):
        return zipfile.ZipInfo(name, date_time=(2020, 1, 1, 0, 0, 0))

    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr(entry("mission"), "mission = " + lua_value(mission) + "\n", zipfile.ZIP_DEFLATED)
        for n, data in sorted(others.items()):
            z.writestr(entry(n), data, zipfile.ZIP_DEFLATED)
    print(f"wrote {out} (stage {stage})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
