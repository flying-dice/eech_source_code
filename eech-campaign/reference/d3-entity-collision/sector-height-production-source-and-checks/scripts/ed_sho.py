def patch(p, pairs):
    d = open(p, "rb").read().decode("utf-8")
    for old, new in pairs:
        assert d.count(old) == 1, (p, old[:70], d.count(old))
        d = d.replace(old, new)
    open(p, "wb").write(d.encode("utf-8"))

patch("eech_sho.c", [
("""typedef char sho_city_words_fit [(CAMPAIGN_CITY_STATE_WORDS <= 10) ? 1 : -1];
""", """/* the observer records the first ten words of a city statement (the position, the box and the sub type); the shape word the host's contract adds after them (CAMPAIGN_CITY_SHAPE, Task49) is the host's own input and is not
   one the observer records. This instrument observes the ORIGINAL profile's campaign-held word: in a hosted build the original raise, getter and pack lines it hooks are compiled out */
#define SHO_CITY_WORDS 10
typedef char sho_city_words_fit [((CAMPAIGN_CITY_STATE_WORDS >= SHO_CITY_WORDS) && (CAMPAIGN_CITY_STATE_WORDS <= SHO_CITY_WORDS + 1)) ? 1 : -1];
"""),
("""	uint32_t words [CAMPAIGN_CITY_STATE_WORDS];
	int i;

	for (i = 0; i < CAMPAIGN_CITY_STATE_WORDS; i++)
	{
		words [i] = bits (&city [i]);
	}

	eech_sho_core_statement ((int) get_local_entity_index ((entity *) en), (int) ENTITY_TYPE_CITY, SHO_FAMILY_CITY, sector_placed, key, words, CAMPAIGN_CITY_STATE_WORDS);""",
"""	uint32_t words [SHO_CITY_WORDS];
	int i;

	for (i = 0; i < SHO_CITY_WORDS; i++)
	{
		words [i] = bits (&city [i]);
	}

	eech_sho_core_statement ((int) get_local_entity_index ((entity *) en), (int) ENTITY_TYPE_CITY, SHO_FAMILY_CITY, sector_placed, key, words, SHO_CITY_WORDS);"""),
])
patch("eech_sho_funnel_check.c", [
("""	uint32_t words[CAMPAIGN_CITY_STATE_WORDS];
	int i;

	for (i = 0; i < CAMPAIGN_CITY_STATE_WORDS; i++)
	{
		words[i] = word_bits (&city[i]);
	}

	eech_sho_core_statement (e->slot, CHECK_CITY_TYPE, FAMILY_CITY, sector_placed, key, words, CAMPAIGN_CITY_STATE_WORDS);""",
"""	uint32_t words[10];
	int i;

	/* the observer records the first ten words of a city statement (eech_sho.c, SHO_CITY_WORDS) */
	for (i = 0; i < 10; i++)
	{
		words[i] = word_bits (&city[i]);
	}

	eech_sho_core_statement (e->slot, CHECK_CITY_TYPE, FAMILY_CITY, sector_placed, key, words, 10);"""),
])
print("ok")
