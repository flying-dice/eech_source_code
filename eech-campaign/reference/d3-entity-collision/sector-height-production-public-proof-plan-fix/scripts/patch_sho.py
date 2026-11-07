import sys

p = sys.argv[1] + "/eech-campaign/crates/eech-engine-sys/csrc/eech_sho.c"
d = open(p, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, old[:70]
    d = d.replace(old, new)


rep("""	uint32_t tallest, terrain, after_tallest, final_word;
	int final_seen;
};""", """	uint32_t tallest, terrain, after_tallest, final_word;
	int final_seen;
	/* the consumed word was NOT captured (a hosted build: the getter's value is the Host's, returned inside the original statement, and the retired member is not read): the report says so instead of printing a word */
	int uncaptured;
};""")
rep("""static int pending_clearance = -1;
""", """static int pending_clearance = -1;
static int next_uncaptured;
""")
rep("""void eech_sho_core_clearance_tallest (int sx, int sz, uint32_t tallest, uint32_t terrain, uint32_t after_tallest)
{
	seq++;
""", """void eech_sho_core_clearance_tallest (int sx, int sz, uint32_t tallest, uint32_t terrain, uint32_t after_tallest)
{
	int uncaptured = next_uncaptured;

	next_uncaptured = 0;

	seq++;
""")
rep("""	clearances [pending_clearance].tallest = tallest;
""", """	clearances [pending_clearance].tallest = tallest;

	clearances [pending_clearance].uncaptured = uncaptured;
""")
rep("""	sector *raw = (sector *) get_local_entity_data ((entity *) sector_entity);

	eech_sho_core_clearance_tallest ((int) raw->x_sector, (int) raw->z_sector, bits (&raw->tallest_structure_height), bits (terrain_elevation), bits (min_height));""", """	sector *raw = (sector *) get_local_entity_data ((entity *) sector_entity);

#ifdef EECH_CAMPAIGN_HOST_ENTITY
	/* HOSTED: the word the clearance consumed is the Host's, returned by the sector getter INSIDE the original statement this hook follows. The legacy member is retired storage and a second query would not be the value
	   the original consumed, so the word is NOT captured: the record keeps the sector, the terrain elevation and the min_height AFTER the statement (which carries the consumed word's effect) and says `tallest=uncaptured` */
	next_uncaptured = 1;

	eech_sho_core_clearance_tallest ((int) raw->x_sector, (int) raw->z_sector, 0, bits (terrain_elevation), bits (min_height));
#else
	eech_sho_core_clearance_tallest ((int) raw->x_sector, (int) raw->z_sector, bits (&raw->tallest_structure_height), bits (terrain_elevation), bits (min_height));
#endif""")
rep("""	put (&o, "sho-clearance i=%d seq=%llu sx=%d sz=%d tallest=0x%08x terrain=0x%08x after_tallest=0x%08x final=0x%08x final_seen=%d\\n",
		index, U (c->seq), c->sx, c->sz, (unsigned) c->tallest, (unsigned) c->terrain, (unsigned) c->after_tallest, (unsigned) c->final_word, c->final_seen);""", """	if (c->uncaptured)
	{
		put (&o, "sho-clearance i=%d seq=%llu sx=%d sz=%d tallest=uncaptured terrain=0x%08x after_tallest=0x%08x final=0x%08x final_seen=%d\\n",
			index, U (c->seq), c->sx, c->sz, (unsigned) c->terrain, (unsigned) c->after_tallest, (unsigned) c->final_word, c->final_seen);
	}
	else
	{
		put (&o, "sho-clearance i=%d seq=%llu sx=%d sz=%d tallest=0x%08x terrain=0x%08x after_tallest=0x%08x final=0x%08x final_seen=%d\\n",
			index, U (c->seq), c->sx, c->sz, (unsigned) c->tallest, (unsigned) c->terrain, (unsigned) c->after_tallest, (unsigned) c->final_word, c->final_seen);
	}""")
open(p, "wb").write(d.encode("utf-8"))
print("ok")
