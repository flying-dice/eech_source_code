/* EECH headless build: the MSVC <io.h> directory search the EECH sources use (implemented in csrc/eech_posix.c) */
#ifndef EECH_COMPAT_IO_H
#define EECH_COMPAT_IO_H
#include "windows.h"

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
