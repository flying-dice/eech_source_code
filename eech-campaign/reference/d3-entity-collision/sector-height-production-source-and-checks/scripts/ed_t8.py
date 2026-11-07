p = "entity_authority.rs"
d = open(p, "rb").read().decode("utf-8")
old = """        ("roster_after", 2),
        ("altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
}

#[test]
fn each_single_fault_of_the_sector_height_grammar"""
new = """        ("roster_after", 2),
        ("candidate_altered", 0),
    ] {
        assert_eq!(row[field], expected, "{field}: {row}");
    }
}

#[test]
fn each_single_fault_of_the_sector_height_grammar"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
