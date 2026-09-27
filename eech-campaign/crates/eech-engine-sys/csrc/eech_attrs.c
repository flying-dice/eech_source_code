/*
 * Patch P1 (64-bit blocker B1; eech-campaign docs/patches.md, docs/64-bit.md),
 * the same marshaller as eech-sys/csrc/eech_attrs.c.
 *
 * en_creat.c :: create_local_entity / create_client_server_entity take the
 * entity attributes as variadic arguments and hand them on as a `char *`
 * list that get_list_item () walks. On i386 that list IS the argument stack,
 * so the original reinterprets the va_list: `pargs_buffer = (char *) pargs`.
 * On x86-64 a va_list is a descriptor (System V) or has 8-byte slots (Win64),
 * so the marshaller walks the attribute grammar with va_arg and writes the
 * list the readers expect: each item at sizeof (its read type), promoted as
 * the variadic call promoted it (char -> int, float -> double).
 */

#include "project.h"

#include "eech_attrs.h"


#define PUT(TYPE, VALUE) \
	do \
	{ \
		if ((size_t) (cursor - buffer) + sizeof (TYPE) > size) \
		{ \
			debug_fatal ("the marshalled entity attribute list exceeds %lu bytes", (unsigned long) size); \
		} \
		quick_set_list_item (cursor, TYPE, (VALUE)); \
	} \
	while (0)

char *eech_marshal_stack_attributes (char *buffer, size_t size, va_list pargs)
{
	char
		*cursor = buffer;

	entity_attributes
		attr;

	while (TRUE)
	{
		/* enums are passed as int */
		attr = (entity_attributes) va_arg (pargs, int);

		PUT (entity_attributes, attr);

		switch (attr)
		{
			case entity_attr_end:
			{
				return buffer;
			}
			case entity_attr_attitude_angles:
			{
				/* ENTITY_ATTR_ATTITUDE_ANGLES: three doubles */
				PUT (double, va_arg (pargs, double));
				PUT (double, va_arg (pargs, double));
				PUT (double, va_arg (pargs, double));

				break;
			}
			case entity_attr_char_value:
			{
				/* ENTITY_ATTR_CHAR_VALUE: char_types, char promoted to int (read as int) */
				PUT (char_types, (char_types) va_arg (pargs, int));
				PUT (int, va_arg (pargs, int));

				break;
			}
			case entity_attr_child_pred:
			case entity_attr_parent:
			{
				PUT (list_types, (list_types) va_arg (pargs, int));
				PUT (entity *, va_arg (pargs, entity *));

				break;
			}
			case entity_attr_float_value:
			{
				/* float promoted to double (read as double) */
				PUT (float_types, (float_types) va_arg (pargs, int));
				PUT (double, va_arg (pargs, double));

				break;
			}
			case entity_attr_int_value:
			{
				PUT (int_types, (int_types) va_arg (pargs, int));
				PUT (int, va_arg (pargs, int));

				break;
			}
			case entity_attr_ptr_value:
			{
				PUT (ptr_types, (ptr_types) va_arg (pargs, int));
				PUT (void *, va_arg (pargs, void *));

				break;
			}
			case entity_attr_string:
			{
				PUT (string_types, (string_types) va_arg (pargs, int));
				PUT (const char *, va_arg (pargs, const char *));

				break;
			}
			case entity_attr_vec3d:
			{
				/* three floats promoted to doubles */
				PUT (vec3d_types, (vec3d_types) va_arg (pargs, int));
				PUT (double, va_arg (pargs, double));
				PUT (double, va_arg (pargs, double));
				PUT (double, va_arg (pargs, double));

				break;
			}
			default:
			{
				/* en_attrs.c :: set_local_entity_attributes' own default */
				debug_fatal ("Invalid entity attribute = %d", (int) attr);
			}
		}
	}
}

