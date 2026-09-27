/*
 * Patch P1 (64-bit blocker B1, docs/patches.md, docs/64-bit.md).
 *
 * en_creat.c :: create_local_entity / create_client_server_entity take the
 * entity attributes as variadic arguments (en_attrs.h ENTITY_ATTR_* macros)
 * and hand them to the creation functions as a `char *` list that
 * get_list_item () walks (en_attrs.c :: set_local_entity_attributes,
 * pack_entity_attributes). On i386 that list IS the argument stack, so the
 * original simply reinterprets the va_list: `pargs_buffer = (char *) pargs`.
 *
 * On x86-64 System V a va_list is a descriptor of register save areas, and on
 * Win64 every variadic slot is 8 bytes, so the reinterpretation reads garbage
 * (eech_k_probe_va_list_reinterpretation demonstrates it). The marshaller
 * walks the same grammar with va_arg and writes exactly the list the readers
 * expect: each item at sizeof (its read type), in order, promoted as the
 * variadic call promoted it (char -> int, float -> double).
 */

#include "project.h"

#include "eech_kernel_internal.h"

#define PUT(TYPE, VALUE) \
	do \
	{ \
		if ((size_t) (cursor - buffer) + sizeof (TYPE) > size) \
		{ \
			eech_abort (EECH_STATUS_FATAL, "entity attribute list too long", "the marshalled attribute list exceeds %lu bytes", (unsigned long) size); \
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
				eech_abort (EECH_STATUS_FATAL, "Invalid entity attribute = %d", "Invalid entity attribute = %d", (int) attr);
			}
		}
	}
}

/*
 * Probes of blocker B1 (tests only). Both are called as
 * probe (read_back, count, ENTITY_ATTR_INT_VALUE (t, v), ..., ENTITY_ATTR_END)
 * and read `count` int attributes back through get_list_item, as
 * set_local_entity_attributes does, into read_back as (type, value) pairs.
 * They return 1 when the list read back ends with entity_attr_end where it
 * should.
 */
static int read_int_attributes (char *pargs_buffer, int *read_back, int count)
{
	int
		i;

	for (i = 0; i < count; i++)
	{
		if (get_list_item (pargs_buffer, entity_attributes) != entity_attr_int_value)
		{
			return 0;
		}

		read_back[i * 2] = get_list_item (pargs_buffer, int_types);
		read_back[i * 2 + 1] = get_list_item (pargs_buffer, int);
	}

	return get_list_item (pargs_buffer, entity_attributes) == entity_attr_end;
}

/* the original en_creat.c line, verbatim: pargs_buffer = (char *) pargs */
int eech_k_probe_va_list_reinterpretation (int *read_back, int count, ...)
{
	va_list
		pargs;

	char
		*pargs_buffer;

	int
		ok;

	memset (read_back, 0, sizeof (int) * 2 * (size_t) count);

	va_start (pargs, count);

	pargs_buffer = (char *) pargs;

	/* reading the reinterpreted list may run past what was passed: read only
	   what the probe owns (at most the descriptor on x86-64) */
	if (sizeof (va_list) < (size_t) (4 + 8 * count) && sizeof (va_list) != sizeof (char *))
	{
		/* x86-64 SysV: va_list is a 24-byte descriptor; read its first words as
		   the original would, within the descriptor */
		int *words = (int *) pargs_buffer;

		read_back[0] = words[0];
		read_back[1] = words[1];

		ok = 0;
	}
	else
	{
		ok = read_int_attributes (pargs_buffer, read_back, count);
	}

	va_end (pargs);

	return ok;
}

int eech_k_probe_marshalled (int *read_back, int count, ...)
{
	va_list
		pargs;

	char
		buffer[EECH_STACK_ATTRIBUTES_SIZE];

	int
		ok;

	jmp_buf
		*saved = eech_abort_point;

	memset (read_back, 0, sizeof (int) * 2 * (size_t) count);

	va_start (pargs, count);

	eech_marshal_stack_attributes (buffer, sizeof (buffer), pargs);

	ok = read_int_attributes (buffer, read_back, count);

	va_end (pargs);

	eech_abort_point = saved;

	return ok;
}
