/* EECH headless build: patch P1 (64-bit blocker B1), see eech_attrs.c */
#ifndef EECH_ATTRS_H
#define EECH_ATTRS_H

#include <stdarg.h>
#include <stddef.h>

#define EECH_STACK_ATTRIBUTES_SIZE 1024

char *eech_marshal_stack_attributes (char *buffer, size_t size, va_list pargs);

#endif
