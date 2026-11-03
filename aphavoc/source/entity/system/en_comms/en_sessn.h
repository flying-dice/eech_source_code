

































































#define PACK_ENTIRE_SESSION	0	// premake if change





extern int pack_session (unsigned char *buffer, int *buffer_size, pack_modes mode);

extern int unpack_session (unsigned char *buffer, int buffer_size, pack_modes mode);

extern void initialise_entity_list_validation (void);

extern int register_entity_list_link_for_post_unpack_validation (struct ENTITY *en, int list_type);

extern int register_entity_list_root_for_post_unpack_validation (struct ENTITY *en, int list_type);

/* FIXME: 2nd arg should be of type meta_smoke_list_types, but its declaration is not available yet, which breaks in gcc (yem) */
extern void register_attach_meta_smoke_list_to_object (entity *en, int type, int attachment_point, int *entity_index_list, int count);

extern void register_attach_sound_effect_to_object (entity *en, int index, entity_sides side, entity_sub_types sub_type, int valid, int sample_index);

extern void dump_entities (void);




