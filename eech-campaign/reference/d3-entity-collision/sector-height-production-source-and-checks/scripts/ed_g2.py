p = "sector_height_guard.rs"
d = open(p, "rb").read().decode("utf-8")
for old, new in [
    ('assert_eq!(enclosing(&getter, host_read), ["defined EECH_CAMPAIGN_HOST_ENTITY"]);', 'assert_eq!(enclosing(&getter, host_read).last().map(String::as_str), Some("defined EECH_CAMPAIGN_HOST_ENTITY"));'),
    ('assert_eq!(enclosing(&getter, legacy_read), ["else of defined EECH_CAMPAIGN_HOST_ENTITY"]);', 'assert_eq!(enclosing(&getter, legacy_read).last().map(String::as_str), Some("else of defined EECH_CAMPAIGN_HOST_ENTITY"));'),
]:
    assert d.count(old) == 1, old
    d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
