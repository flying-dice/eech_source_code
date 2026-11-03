

































































enum PACK_MODES
{
	PACK_MODE_SERVER_SESSION,
	PACK_MODE_CLIENT_SESSION,
	PACK_MODE_BROWSE_SESSION,
	PACK_MODE_UPDATE_ENTITY,
	NUM_PACK_MODES
};

typedef enum PACK_MODES pack_modes;





struct PACK_BUFFER
{
	unsigned char
		*buffer,
		*ptr;

	int
		size,
		bit_counter,
		overflow;
};

typedef struct PACK_BUFFER pack_buffer;





struct UNPACK_BUFFER
{
	unsigned char
		*buffer,
		*ptr;

	int
		size,
		bit_counter,
		overflow;
};

typedef struct UNPACK_BUFFER unpack_buffer;





extern void (*fn_pack_local_entity_data[NUM_ENTITY_TYPES]) (entity *en, pack_modes mode);

#define pack_local_entity_data(EN,MODE) (fn_pack_local_entity_data[get_local_entity_type ((EN))] ((EN), (MODE)))





extern void (*fn_unpack_local_entity_data[NUM_ENTITY_TYPES]) (entity *en, entity_types type, pack_modes mode);





extern void initialise_entity_pack_default_functions (void);

extern void unpack_local_entity_data (entity *en, entity_types type, pack_modes mode);

extern void unpack_client_server_entity_data (entity *en);





extern pack_buffer
	*current_pack_buffer;

#define get_pack_buffer() (current_pack_buffer)

#define get_pack_buffer_overflow() (current_pack_buffer->overflow)





extern void open_pack_buffer (unsigned char *buffer, int buffer_size);

extern int close_pack_buffer (void);

extern int get_current_pack_buffer_bit_size (void);

extern void pack_signed_data (int unpacked_data, int number_of_bits_to_pack);

extern void pack_unsigned_data (unsigned int unpacked_data, int number_of_bits_to_pack);





extern unpack_buffer
	*current_unpack_buffer;

#define get_unpack_buffer() (current_unpack_buffer)

#define get_unpack_buffer_overflow() (current_unpack_buffer->overflow)





extern void open_unpack_buffer (unsigned char *buffer, int buffer_size);

extern void close_unpack_buffer (void);

extern int unpack_signed_data (int number_of_bits_to_unpack);

extern unsigned int unpack_unsigned_data (int number_of_bits_to_unpack);

extern int get_current_pack_buffer_size (void);





extern int
	signed_int_pack_stats [33],
	unsigned_int_pack_stats [33];

extern void debug_print_int_pack_stats (void);




