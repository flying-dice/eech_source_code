#!/usr/bin/env python3
# Static Lua ABI check for hosting eech_dc.dll in DCS World: compares the
# module's PE import table with a host lua.dll's PE export table, and lists
# both images' DLL dependencies and the host Lua's error-unwinding imports.
# Reads files only; it loads and runs nothing.
#
#   python -I pe-lua-abi.py <eech_dc.dll> <host lua.dll>
#
# Exit status 0 only when every printed check passes: an x86-64 PE32+ module
# that exports luaopen_eech_dc, has no delay imports (they are not read) and no
# MinGW runtime DLL imports, a PE32+ host Lua, and every symbol the module
# imports from the host Lua's DLL name exported by it. 1 otherwise; 2 on
# unreadable input. The unwinding imports are reported, not judged.
#
# PASS is link-level only. It does not show ABI compatibility: the candidate
# passes against DCS lua.dll, yet DCS's LUA_IDSIZE 256 lua_Debug (320 bytes)
# is overrun by mlua-sys 0.6.8's 60/120-byte assumption on the first native
# error (reference/m5-dcs-probes/README.md).
import struct
import sys


def image(path):
    with open(path, "rb") as f:
        b = f.read()
    pe = struct.unpack_from("<I", b, 0x3C)[0]
    if b[pe:pe + 4] != b"PE\0\0":
        raise ValueError(f"{path}: not a PE image")
    machine, nsec = struct.unpack_from("<HH", b, pe + 4)
    optsz = struct.unpack_from("<H", b, pe + 20)[0]
    opt = pe + 24
    wide = struct.unpack_from("<H", b, opt)[0] == 0x20B
    dirs = opt + (112 if wide else 96)
    secs = []
    for i in range(nsec):
        s = opt + optsz + 40 * i
        vsize, va, rsize, raw = struct.unpack_from("<IIII", b, s + 8)
        secs.append((va, max(vsize, rsize), raw))
    return b, machine, wide, dirs, secs


def offset(secs, rva):
    for va, size, raw in secs:
        if va <= rva < va + size:
            return rva - va + raw
    raise ValueError(f"RVA {rva:#x} is in no section")


def cstr(b, o):
    return b[o:b.index(b"\0", o)].decode("ascii", "replace")


def directory(b, dirs, index):
    return struct.unpack_from("<II", b, dirs + 8 * index)


def exports(path):
    b, _, _, dirs, secs = image(path)
    rva, _ = directory(b, dirs, 0)
    if not rva:
        return "", set()
    o = offset(secs, rva)
    name = struct.unpack_from("<I", b, o + 12)[0]
    count = struct.unpack_from("<I", b, o + 24)[0]
    names = offset(secs, struct.unpack_from("<I", b, o + 32)[0])
    symbols = {cstr(b, offset(secs, struct.unpack_from("<I", b, names + 4 * i)[0])) for i in range(count)}
    return cstr(b, offset(secs, name)), symbols


def imports(path):
    b, machine, wide, dirs, secs = image(path)
    rva, _ = directory(b, dirs, 1)
    delay, _ = directory(b, dirs, 13)
    result = {}
    fmt, size, flag = ("<Q", 8, 1 << 63) if wide else ("<I", 4, 1 << 31)
    o = offset(secs, rva) if rva else None
    while o is not None:
        lookup, _, _, name, address = struct.unpack_from("<IIIII", b, o)
        if name == 0:
            break
        symbols = result.setdefault(cstr(b, offset(secs, name)), [])
        t = offset(secs, lookup or address)
        while True:
            v = struct.unpack_from(fmt, b, t)[0]
            if v == 0:
                break
            symbols.append(f"#{v & 0xFFFF}" if v & flag else cstr(b, offset(secs, v & 0x7FFFFFFF) + 2))
            t += size
        o += 20
    return machine, wide, delay != 0, result


def main(argv):
    if len(argv) != 3:
        print("usage: pe-lua-abi.py <eech_dc.dll> <host lua.dll>", file=sys.stderr)
        return 2
    module, host = argv[1], argv[2]
    try:
        machine, wide, delayed, needed = imports(module)
        host_name, provided = exports(host)
        _, host_wide, host_delayed, host_needs = imports(host)
        _, module_exports = exports(module)
    except (OSError, ValueError, struct.error) as e:
        print(f"error: {e}", file=sys.stderr)
        return 2
    print(f"module: {module}")
    print(f"  machine {machine:#06x}, PE32{'+' if wide else ''}, delay imports {'present' if delayed else 'none'}")
    print(f"  exports luaopen_eech_dc: {'yes' if 'luaopen_eech_dc' in module_exports else 'NO'}")
    print("  imported DLLs: " + ", ".join(sorted(needed)))
    print(f"host Lua: {host} (export name {host_name or '-'})")
    print(f"  PE32{'+' if host_wide else ''}, {len(provided)} exports, delay imports {'present' if host_delayed else 'none'}")
    print("  imported DLLs: " + ", ".join(sorted(host_needs)))
    unwinding = sorted(s for syms in host_needs.values() for s in syms
                       if s in ("longjmp", "_longjmp", "_setjmp", "_setjmpex", "_CxxThrowException") or s.startswith("__CxxFrameHandler"))
    print("  error-unwinding imports: " + (", ".join(unwinding) or "none found"))
    lua = [dll for dll in needed if dll.lower() == (host_name or "lua.dll").lower()]
    missing = sorted({s for dll in lua for s in needed[dll] if s not in provided})
    for dll in lua:
        print(f"{dll}: {len(needed[dll])} symbols imported, {len(provided)} host exports")
    for s in missing:
        print(f"  missing {s}")
    mingw = sorted(d for d in needed if d.lower().startswith(("libgcc", "libwinpthread", "libstdc++")))
    checks = [
        ("module is x86-64 PE32+", machine == 0x8664 and wide),
        ("host Lua is PE32+", host_wide),
        ("module exports luaopen_eech_dc", "luaopen_eech_dc" in module_exports),
        ("module has no delay imports", not delayed),
        (f"module imports from {host_name or 'lua.dll'}", bool(lua)),
        ("no imported Lua symbol is missing from the host", bool(lua) and not missing),
        ("module imports no MinGW runtime DLL", not mingw),
    ]
    for label, ok in checks:
        print(f"{'pass' if ok else 'FAIL'}: {label}")
    status = 0 if all(ok for _, ok in checks) else 1
    # link level only: struct layouts (lua_Debug, LUA_IDSIZE) are not visible in PE tables
    print("PASS (link level; struct layouts not checked)" if status == 0 else "FAIL")
    return status

if __name__ == "__main__":
    sys.exit(main(sys.argv))
