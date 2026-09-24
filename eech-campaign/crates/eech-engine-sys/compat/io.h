/* EECH headless build: the MSVC <io.h> directory search the EECH sources use (implemented in csrc/eech_win32.c) */
#ifndef EECH_COMPAT_IO_H
#define EECH_COMPAT_IO_H
#include "windows.h"

#ifdef _WIN32
/*
 * MinGW-w64: the CRT's own <io.h> first (its dirent.h, fcntl.h and unistd.h
 * need it), then EECH's _findfirst family over it: the CRT's are macros for
 * its 32/64-bit time variants, and EECH's are renamed eech_w32_*
 * (eech_win32_names.h).
 */
#include_next <io.h>
#undef _finddata_t
#undef _findfirst
#define _findfirst eech_w32__findfirst
#undef _findnext
#define _findnext eech_w32__findnext
#endif

#define _A_NORMAL 0x00
#define _A_RDONLY 0x01
#define _A_HIDDEN 0x02
#define _A_SYSTEM 0x04
#define _A_SUBDIR 0x10
#define _A_ARCH 0x20

struct _finddata_t
{
	unsigned attrib;
	int64_t time_create, time_access, time_write;
	uint32_t size;
	char name[260];
};

intptr_t _findfirst (const char *filespec, struct _finddata_t *fileinfo);
int _findnext (intptr_t handle, struct _finddata_t *fileinfo);
int _findclose (intptr_t handle);
#endif
