import pathlib, sys
raw = pathlib.Path(sys.argv[1]).read_bytes().decode("utf-8")
b, e = "-----BEGIN SHO REPORT-----\n", "-----END SHO REPORT-----"
s = raw.index(b) + len(b)
t = raw.index(e, s)
pathlib.Path(sys.argv[2]).write_bytes(raw[s:t].encode("utf-8"))
print(t - s)
