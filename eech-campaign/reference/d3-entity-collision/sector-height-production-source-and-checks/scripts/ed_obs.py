def patch(p, pairs):
    d = open(p, "rb").read().decode("utf-8")
    for old, new in pairs:
        assert d.count(old) == 1, (p, old[:70], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))

patch("sector_height_observer.rs", [(
"""    assert_eq!(checks, 179, "the control made {checks} expectations, not the 179 of the unhooked run");""",
"""    // the observer hooks the ORIGINAL body of the response, the getter and the pack, which a hosted build compiles out (Task49, the host-owned sector height): this control runs in the original-solver profile (feature
    // `collision-fixture`), where the accepted 179 expectations hold unchanged and the host's raise expression is run beside the original response over every domain case (42 more)
    assert_eq!(checks, 179 + 42, "the control made {checks} expectations, not the 179 of the unhooked run and the 42 of the differential");""")])

patch("sector_height.rs", [(
"""    println!("sector height expectations made: {checks}");
    assert!(checks >= 150, "only {checks} expectations were made: the control did not run in full");""",
"""    println!("sector height expectations made: {checks}");
    // the two profiles of the control (see the header of csrc/eech_sector_height_check.c): the original-solver profile (feature `collision-fixture`) runs the accepted 179 expectations of the original response, getter and pack
    // unchanged and 42 more (the host's raise expression run beside the original response over every domain case and both rounding modes); the hosted profile (the default) runs the 150 expectations of the host-owned word
    let wanted = if cfg!(feature = "collision-fixture") { 179 + 42 } else { 150 };
    assert_eq!(checks, wanted, "the control made {checks} expectations, not the {wanted} of its profile: a group of cases did not run");""")])
print("ok")
