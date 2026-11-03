

































































#ifndef IMAPS_H
#define IMAPS_H

enum IMAP_TYPES
{
	IMAP_IMPORTANCE,
	IMAP_BASE_DISTANCE,
	IMAP_AIR_DEFENCE,
	IMAP_SURFACE_DEFENCE,

	NUM_IMAP_TYPES
};

typedef enum IMAP_TYPES imap_types;





extern void initialise_imaps (void);

extern void deinitialise_imaps (void);

extern float get_imap_value (imap_types type, entity_sides side, int x, int z);

extern unsigned char get_imap_char_value (imap_types type, entity_sides side, int x, int z);



extern void update_imap_sector_side (entity *en, int in_use);



extern void update_imap_importance_level (entity *en, int in_use);

extern void normalise_importance_imaps (void);



extern void update_keysite_distance_to_friendly_base (entity *en, entity_sides side);

extern void update_imap_distance_to_friendly_base (entity_sides side);

extern void normalise_base_distance_imaps (void);



extern void update_imap_surface_to_air_defence_level (entity *en, entity *sector, int in_use);

extern void normalise_air_defence_imaps (void);



extern void update_imap_surface_to_surface_defence_level (entity *en, entity *sector, int in_use);

extern void normalise_surface_defence_imaps (void);









#endif // IMAPS_H
