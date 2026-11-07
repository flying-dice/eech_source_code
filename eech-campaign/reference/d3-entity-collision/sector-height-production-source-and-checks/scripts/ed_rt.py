p = "src/roster.rs"
d = open(p, "rb").read().decode("utf-8")
old = """        assert!(!is_fixed_family(ENTITY_CITY));
        assert!(is_fixed_family(ENTITY_CITY_BUILDING));
    }
}
"""
new = """        assert!(!is_fixed_family(ENTITY_CITY));
        assert!(is_fixed_family(ENTITY_CITY_BUILDING));
    }

    #[test]
    fn a_sectors_height_is_the_zero_word_until_an_arrival_or_a_restore_writes_it_and_a_clear_ends_every_word() {
        let mut roster = SectorRoster::default();
        assert_eq!(roster.sector_height((3, 4)), 0, "a sector the roster holds no word for is the zero fill of a new sector");
        let key = roster.insert((3, 4), 17, None);
        assert_eq!(roster.awaiting_arrival(key), Some((3, 4)), "the arrival is still to come at the insertion");
        roster.raise_sector_height(key, Some(0x42ca_0000)).expect("the arrival");
        assert_eq!(roster.sector_height((3, 4)), 0x42ca_0000);
        assert_eq!(roster.sector_height((4, 3)), 0, "the word belongs to its own sector");
        assert_eq!(roster.awaiting_arrival(key), None, "the arrival is taken once");
        assert!(roster.raise_sector_height(key, Some(0x4348_0000)).unwrap_err().0.contains("already arrived"));
        assert_eq!(roster.sector_height((3, 4)), 0x42ca_0000, "a refused second raise changes nothing");
        // an arrival that leaves the word (None) still takes the arrival
        let lower = roster.insert((3, 4), 17, None);
        roster.raise_sector_height(lower, None).expect("an arrival that raises nothing");
        assert_eq!(roster.awaiting_arrival(lower), None);
        assert_eq!(roster.sector_height((3, 4)), 0x42ca_0000);
        // a removal never lowers it and a key that ended is no arrival
        roster.remove(key).expect("removed");
        assert_eq!(roster.sector_height((3, 4)), 0x42ca_0000);
        assert!(roster.raise_sector_height(key, Some(0)).unwrap_err().0.contains("has ended"));
        assert_eq!(roster.awaiting_arrival(key), None);
        // a restore states ANY raw word, and the clear of a boot ends every word
        for word in [0x8000_0000u32, 0xff80_0000, 0x7f80_0000, 0x7fc0_f00d, 0xbf80_0000, 0] {
            roster.restore_sector_height((3, 4), word);
            assert_eq!(roster.sector_height((3, 4)), word);
        }
        roster.restore_sector_height((9, 9), 0x3f80_0000);
        roster.clear();
        assert_eq!((roster.sector_height((3, 4)), roster.sector_height((9, 9))), (0, 0));
    }

    #[test]
    fn a_city_childs_key_names_no_sector_and_never_awaits_an_arrival() {
        let mut roster = SectorRoster::default();
        let city = roster.insert((0, 0), ENTITY_CITY, None);
        let child = roster.child_insert(city, None, ENTITY_CITY_BUILDING).expect("listed");
        assert_eq!(roster.awaiting_arrival(child), None, "a building's place in its city's list is not an arrival in a sector");
        assert!(roster.raise_sector_height(child, Some(1)).is_err());
        assert_eq!(roster.awaiting_arrival(99), None, "a key never issued");
    }
}
"""
assert d.count(old) == 1
d = d.replace(old, new)
open(p, "wb").write(d.encode("utf-8"))
print("ok")
