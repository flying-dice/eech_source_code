/*
 * EECH headless build: the Win32 and MSVC CRT surface EECH calls, over POSIX.
 *
 * Paths: EECH writes Windows paths ("..\\common\\maps\\map6\\terrain"). Every
 * path taken here is converted to '/' and resolved case-insensitively, one
 * component at a time, because the data files are named in mixed case.
 *
 * There is no window, no registry and no multimedia device: those calls
 * succeed as no-ops or report "not present", which is what EECH's own code
 * handles (a missing registry key, no CD, no timer device).
 */

#define _GNU_SOURCE
#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <time.h>
#ifdef _WIN32
/* MinGW-w64: no mmap or fnmatch (see map_private, map_anonymous, match_name) */
#define O_EECH_BINARY O_BINARY
#define PROT_READ 1
#define PROT_WRITE 2
#else
#include <fnmatch.h>
#include <sys/mman.h>
#define O_EECH_BINARY 0
#endif

#include "windows.h"
#include "io.h"
#include "winsock.h"

#include "eech_headless.h"

static __thread DWORD last_error;

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* paths */

static void resolve_component (char *dir_end, char *path)
{
	/* path is NUL-terminated at the end of the component that starts after dir_end */
#ifdef _WIN32
	/* Windows file systems resolve names case-insensitively themselves */
	(void) dir_end;
	(void) path;
	return;
#endif
	struct stat st;
	if (stat (path, &st) == 0)
	{
		return;
	}
	char *name = dir_end ? dir_end + 1 : path;
	char dir[PATH_MAX];
	if (dir_end)
	{
		size_t n = (size_t) (dir_end - path);
		if (n == 0)
		{
			strcpy (dir, "/");
		}
		else
		{
			memcpy (dir, path, n);
			dir[n] = 0;
		}
	}
	else
	{
		strcpy (dir, ".");
	}
	DIR *d = opendir (dir);
	if (!d)
	{
		return;
	}
	struct dirent *e;
	while ((e = readdir (d)) != NULL)
	{
		if (strcasecmp (e->d_name, name) == 0)
		{
			memcpy (name, e->d_name, strlen (name));
			break;
		}
	}
	closedir (d);
}

void eech_native_path (const char *in, char *out, size_t size)
{
	size_t i;
	for (i = 0; in[i] && i + 1 < size; i++)
	{
		out[i] = (in[i] == '\\') ? '/' : in[i];
	}
	out[i] = 0;
	/* collapse doubled separators */
	char *w = out;
	for (char *r = out; *r; r++)
	{
		if (!(r[0] == '/' && r[1] == '/'))
		{
			*w++ = *r;
		}
	}
	*w = 0;
	/* resolve each component case-insensitively */
	char *prev = NULL;
	for (char *p = out; ; p++)
	{
		if (*p == '/' || *p == 0)
		{
			char c = *p;
			if (p != out)
			{
				*p = 0;
				resolve_component (prev, out);
				*p = c;
			}
			if (c == 0)
			{
				break;
			}
			prev = p;
		}
	}
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* MSVC CRT */

char *strupr (char *s)
{
	for (char *p = s; *p; p++)
	{
		*p = (char) toupper ((unsigned char) *p);
	}
	return s;
}

char *strlwr (char *s)
{
	for (char *p = s; *p; p++)
	{
		*p = (char) tolower ((unsigned char) *p);
	}
	return s;
}

char *itoa (int value, char *buffer, int radix)
{
	char tmp[40];
	int i = 0, negative = (radix == 10 && value < 0);
	unsigned int v = negative ? (unsigned int) -value : (unsigned int) value;
	do
	{
		int d = (int) (v % (unsigned int) radix);
		tmp[i++] = (char) (d < 10 ? '0' + d : 'a' + d - 10);
		v /= (unsigned int) radix;
	}
	while (v);
	char *o = buffer;
	if (negative)
	{
		*o++ = '-';
	}
	while (i)
	{
		*o++ = tmp[--i];
	}
	*o = 0;
	return buffer;
}

/* a name matches a find pattern, case-insensitively */
#ifdef _WIN32
static int match_name (const char *pattern, const char *name)
{
	/* '*' and '?', which is all a Windows find pattern has */
	for (; *pattern; pattern++, name++)
	{
		if (*pattern == '*')
		{
			for (;; name++)
			{
				if (match_name (pattern + 1, name))
				{
					return 1;
				}
				if (!*name)
				{
					return 0;
				}
			}
		}
		if (!*name || (*pattern != '?' && tolower ((unsigned char) *pattern) != tolower ((unsigned char) *name)))
		{
			return 0;
		}
	}
	return !*name;
}
#else
static int match_name (const char *pattern, const char *name)
{
	return fnmatch (pattern, name, FNM_CASEFOLD) == 0;
}
#endif

/* _findfirst: pattern "dir\\*.ext" (only the last component may contain wildcards) */

struct find_state
{
	DIR *dir;
	char directory[PATH_MAX];
	char pattern[256];
};

static int find_next_entry (struct find_state *s, char *name, size_t name_size, unsigned *attrib, uint32_t *size, time_t *mtime)
{
	struct dirent *e;
	while ((e = readdir (s->dir)) != NULL)
	{
		if (match_name (s->pattern, e->d_name))
		{
			char full[PATH_MAX * 2];
			struct stat st;
			snprintf (full, sizeof (full), "%s/%s", s->directory, e->d_name);
			if (stat (full, &st) != 0)
			{
				continue;
			}
			snprintf (name, name_size, "%s", e->d_name);
			*attrib = S_ISDIR (st.st_mode) ? _A_SUBDIR : _A_NORMAL;
			*size = (uint32_t) st.st_size;
			*mtime = st.st_mtime;
			return 1;
		}
	}
	return 0;
}

static struct find_state *find_open (const char *filespec)
{
	char native[PATH_MAX];
	eech_native_path (filespec, native, sizeof (native));
	struct find_state *s = calloc (1, sizeof (*s));
	if (!s)
	{
		return NULL;
	}
	char *slash = strrchr (native, '/');
	if (slash)
	{
		*slash = 0;
		snprintf (s->directory, sizeof (s->directory), "%s", slash == native ? "/" : native);
		snprintf (s->pattern, sizeof (s->pattern), "%s", slash + 1);
	}
	else
	{
		strcpy (s->directory, ".");
		snprintf (s->pattern, sizeof (s->pattern), "%s", native);
	}
	/* "*.*" matches every name on Windows */
	if (strcmp (s->pattern, "*.*") == 0)
	{
		strcpy (s->pattern, "*");
	}
	s->dir = opendir (s->directory);
	if (!s->dir)
	{
		free (s);
		return NULL;
	}
	return s;
}

static void find_close (struct find_state *s)
{
	if (s)
	{
		closedir (s->dir);
		free (s);
	}
}

/*
 * _findfirst handles are small indices, not pointers: EECH keeps them in a
 * long (3dobjdb.c, 3dobjid.c, eechini.c), which on 64-bit Windows (LLP64) is
 * 32 bits and would cut a pointer in half.
 */
static struct find_state **find_handles;
static intptr_t number_of_find_handles;

static intptr_t find_handle_new (struct find_state *s)
{
	for (intptr_t i = 0; i < number_of_find_handles; i++)
	{
		if (!find_handles[i])
		{
			find_handles[i] = s;
			return i + 1;
		}
	}
	struct find_state **grown = realloc (find_handles, (size_t) (number_of_find_handles + 16) * sizeof (*grown));
	if (!grown)
	{
		return -1;
	}
	memset (grown + number_of_find_handles, 0, 16 * sizeof (*grown));
	find_handles = grown;
	find_handles[number_of_find_handles] = s;
	number_of_find_handles += 16;
	return number_of_find_handles - 16 + 1;
}

static struct find_state *find_handle_state (intptr_t handle)
{
	return handle >= 1 && handle <= number_of_find_handles ? find_handles[handle - 1] : NULL;
}

intptr_t _findfirst (const char *filespec, struct _finddata_t *fileinfo)
{
	struct find_state *s = find_open (filespec);
	time_t t;
	if (!s)
	{
		errno = ENOENT;
		return -1;
	}
	if (!find_next_entry (s, fileinfo->name, sizeof (fileinfo->name), &fileinfo->attrib, &fileinfo->size, &t))
	{
		find_close (s);
		errno = ENOENT;
		return -1;
	}
	fileinfo->time_write = fileinfo->time_access = fileinfo->time_create = t;
	intptr_t handle = find_handle_new (s);
	if (handle == -1)
	{
		find_close (s);
		errno = ENOMEM;
	}
	return handle;
}

int _findnext (intptr_t handle, struct _finddata_t *fileinfo)
{
	time_t t;
	struct find_state *s = find_handle_state (handle);
	if (!s || !find_next_entry (s, fileinfo->name, sizeof (fileinfo->name), &fileinfo->attrib, &fileinfo->size, &t))
	{
		return -1;
	}
	fileinfo->time_write = fileinfo->time_access = fileinfo->time_create = t;
	return 0;
}

int _findclose (intptr_t handle)
{
	struct find_state *s = find_handle_state (handle);
	if (s)
	{
		find_close (s);
		find_handles[handle - 1] = NULL;
	}
	return 0;
}

static void unix_to_filetime (time_t t, FILETIME *ft)
{
	uint64_t v = ((uint64_t) t + 11644473600ull) * 10000000ull;
	ft->dwLowDateTime = (DWORD) v;
	ft->dwHighDateTime = (DWORD) (v >> 32);
}

HANDLE FindFirstFile (LPCSTR pattern, WIN32_FIND_DATA *data)
{
	struct find_state *s = find_open (pattern);
	time_t t;
	uint32_t size;
	unsigned attrib;
	if (!s)
	{
		last_error = 2;
		return INVALID_HANDLE_VALUE;
	}
	memset (data, 0, sizeof (*data));
	if (!find_next_entry (s, data->cFileName, sizeof (data->cFileName), &attrib, &size, &t))
	{
		find_close (s);
		last_error = 2;
		return INVALID_HANDLE_VALUE;
	}
	data->dwFileAttributes = (attrib & _A_SUBDIR) ? FILE_ATTRIBUTE_DIRECTORY : FILE_ATTRIBUTE_NORMAL;
	data->nFileSizeLow = size;
	unix_to_filetime (t, &data->ftLastWriteTime);
	return s;
}

BOOL FindNextFile (HANDLE find, WIN32_FIND_DATA *data)
{
	time_t t;
	uint32_t size;
	unsigned attrib;
	memset (data, 0, sizeof (*data));
	if (find == INVALID_HANDLE_VALUE || !find_next_entry ((struct find_state *) find, data->cFileName, sizeof (data->cFileName), &attrib, &size, &t))
	{
		last_error = 18;
		return FALSE;
	}
	data->dwFileAttributes = (attrib & _A_SUBDIR) ? FILE_ATTRIBUTE_DIRECTORY : FILE_ATTRIBUTE_NORMAL;
	data->nFileSizeLow = size;
	unix_to_filetime (t, &data->ftLastWriteTime);
	return TRUE;
}

BOOL FindClose (HANDLE find)
{
	if (find != INVALID_HANDLE_VALUE)
	{
		find_close ((struct find_state *) find);
	}
	return TRUE;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* kernel objects: files, mappings, mutexes, events */

enum object_kind { OBJECT_FILE = 0x46494c45, OBJECT_MAPPING, OBJECT_MUTEX, OBJECT_EVENT };

struct object
{
	enum object_kind kind;
	int fd;
	size_t size;
	pthread_mutex_t mutex;
	pthread_cond_t cond;
	int signalled, manual;
};

static struct object *new_object (enum object_kind kind)
{
	struct object *o = calloc (1, sizeof (*o));
	if (o)
	{
		o->kind = kind;
		o->fd = -1;
		pthread_mutex_init (&o->mutex, NULL);
		pthread_cond_init (&o->cond, NULL);
	}
	return o;
}

static void trace_open (const char *native, const char *mode, const void *result);

HANDLE CreateFile (LPCSTR name, DWORD access, DWORD share, LPSECURITY_ATTRIBUTES security, DWORD disposition, DWORD flags, HANDLE template_file)
{
	char native[PATH_MAX];
	int mode = (access & GENERIC_WRITE) ? ((access & GENERIC_READ) ? O_RDWR : O_WRONLY) : O_RDONLY;
	(void) share;
	(void) security;
	(void) flags;
	(void) template_file;
	switch (disposition)
	{
		case CREATE_NEW: mode |= O_CREAT | O_EXCL; break;
		case CREATE_ALWAYS: mode |= O_CREAT | O_TRUNC; break;
		case OPEN_ALWAYS: mode |= O_CREAT; break;
		default: break;
	}
	eech_native_path (name, native, sizeof (native));
	int fd = open (native, mode | O_EECH_BINARY, 0644);
	trace_open (native, "map", fd < 0 ? NULL : native);
	if (fd < 0)
	{
		last_error = 2;
		return INVALID_HANDLE_VALUE;
	}
	struct object *o = new_object (OBJECT_FILE);
	if (!o)
	{
		close (fd);
		return INVALID_HANDLE_VALUE;
	}
	o->fd = fd;
	return o;
}

BOOL ReadFile (HANDLE file, LPVOID buffer, DWORD size, LPDWORD read_count, LPOVERLAPPED overlapped)
{
	struct object *o = file;
	(void) overlapped;
	ssize_t n = read (o->fd, buffer, size);
	if (read_count)
	{
		*read_count = n > 0 ? (DWORD) n : 0;
	}
	return n >= 0;
}

DWORD GetFileSize (HANDLE file, LPDWORD high)
{
	struct object *o = file;
	struct stat st;
	if (fstat (o->fd, &st) != 0)
	{
		return 0xFFFFFFFFu;
	}
	if (high)
	{
		*high = (DWORD) ((uint64_t) st.st_size >> 32);
	}
	return (DWORD) st.st_size;
}

HANDLE CreateFileMapping (HANDLE file, LPSECURITY_ATTRIBUTES security, DWORD protect, DWORD size_high, DWORD size_low, LPCSTR name)
{
	struct object *f = file;
	(void) security;
	(void) protect;
	(void) name;
	if (file == INVALID_HANDLE_VALUE || !f || f->kind != OBJECT_FILE)
	{
		/* named shared memory (sharedmem.c): not provided headless */
		last_error = 5;
		return NULL;
	}
	struct object *o = new_object (OBJECT_MAPPING);
	if (!o)
	{
		return NULL;
	}
	o->fd = dup (f->fd);
	o->size = ((uint64_t) size_high << 32) | size_low;
	if (o->size == 0)
	{
		struct stat st;
		fstat (o->fd, &st);
		o->size = (size_t) st.st_size;
	}
	return o;
}

/* private, writable views of files and anonymous memory */
#ifdef _WIN32
/* a private view is a copy: it reads the file into zeroed memory, as a
   MAP_PRIVATE view is zero past the end of the file */
static void *map_private (int fd, int64_t offset, size_t length)
{
	char *p = calloc (1, length);
	if (!p || _lseeki64 (fd, offset, SEEK_SET) < 0)
	{
		free (p);
		return NULL;
	}
	for (size_t got = 0; got < length; )
	{
		int n = read (fd, p + got, (unsigned) (length - got < (1u << 30) ? length - got : (1u << 30)));
		if (n <= 0)
		{
			break;
		}
		got += (size_t) n;
	}
	return p;
}

static void *map_anonymous (size_t size)
{
	return calloc (1, size);
}

static void unmap (void *base, size_t size)
{
	(void) size;
	free (base);
}
#else
static void *map_private (int fd, int64_t offset, size_t length)
{
	void *p = mmap (NULL, length, PROT_READ | PROT_WRITE, MAP_PRIVATE, fd, (off_t) offset);
	return p == MAP_FAILED ? NULL : p;
}

static void *map_anonymous (size_t size)
{
	void *p = mmap (NULL, size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE, -1, 0);
	return p == MAP_FAILED ? NULL : p;
}

static void unmap (void *base, size_t size)
{
	munmap (base, size);
}
#endif

/* mapping base -> length, for UnmapViewOfFile */
#define MAX_VIEWS 4096
static struct { void *base; size_t size; } views[MAX_VIEWS];
static pthread_mutex_t views_lock = PTHREAD_MUTEX_INITIALIZER;

LPVOID MapViewOfFile (HANDLE mapping, DWORD access, DWORD offset_high, DWORD offset_low, SIZE_T size)
{
	struct object *o = mapping;
	int64_t offset = (int64_t) (((uint64_t) offset_high << 32) | offset_low);
	size_t length = size ? size : o->size - (size_t) offset;
	if (length == 0)
	{
		length = 1;
	}
	int prot = PROT_READ | ((access & FILE_MAP_WRITE) ? PROT_WRITE : 0);
	/* EECH writes into read-only views in places: map privately, copy-on-write */
	void *p = map_private (o->fd, offset, length);
	(void) prot;
	if (!p)
	{
		last_error = 8;
		return NULL;
	}
	pthread_mutex_lock (&views_lock);
	for (int i = 0; i < MAX_VIEWS; i++)
	{
		if (!views[i].base)
		{
			views[i].base = p;
			views[i].size = length;
			break;
		}
	}
	pthread_mutex_unlock (&views_lock);
	return p;
}

BOOL UnmapViewOfFile (LPCVOID base)
{
	pthread_mutex_lock (&views_lock);
	for (int i = 0; i < MAX_VIEWS; i++)
	{
		if (views[i].base == base)
		{
			unmap (views[i].base, views[i].size);
			views[i].base = NULL;
			pthread_mutex_unlock (&views_lock);
			return TRUE;
		}
	}
	pthread_mutex_unlock (&views_lock);
	return FALSE;
}

BOOL CloseHandle (HANDLE object)
{
	struct object *o = object;
	if (!o || object == INVALID_HANDLE_VALUE)
	{
		return FALSE;
	}
	if (o->fd >= 0)
	{
		close (o->fd);
	}
	pthread_mutex_destroy (&o->mutex);
	pthread_cond_destroy (&o->cond);
	free (o);
	return TRUE;
}

/* VirtualAlloc: reserve and commit in one step (EECH's heap reserves then commits the same range) */
LPVOID VirtualAlloc (LPVOID address, SIZE_T size, DWORD type, DWORD protect)
{
	(void) protect;
	if (address && (type & MEM_COMMIT) && !(type & MEM_RESERVE))
	{
		return address;
	}
	void *p = map_anonymous (size);
	if (!p)
	{
		return NULL;
	}
	pthread_mutex_lock (&views_lock);
	for (int i = 0; i < MAX_VIEWS; i++)
	{
		if (!views[i].base)
		{
			views[i].base = p;
			views[i].size = size;
			break;
		}
	}
	pthread_mutex_unlock (&views_lock);
	return p;
}

BOOL VirtualFree (LPVOID address, SIZE_T size, DWORD type)
{
	(void) size;
	if (type & MEM_RELEASE)
	{
		return UnmapViewOfFile (address);
	}
	return TRUE;
}

HANDLE CreateMutex (LPSECURITY_ATTRIBUTES security, BOOL owner, LPCSTR name)
{
	(void) security;
	(void) name;
	struct object *o = new_object (OBJECT_MUTEX);
	if (o && owner)
	{
		pthread_mutex_lock (&o->mutex);
	}
	return o;
}

BOOL ReleaseMutex (HANDLE mutex)
{
	struct object *o = mutex;
	return o && pthread_mutex_unlock (&o->mutex) == 0;
}

HANDLE CreateEvent (LPSECURITY_ATTRIBUTES security, BOOL manual, BOOL initial, LPCSTR name)
{
	(void) security;
	(void) name;
	struct object *o = new_object (OBJECT_EVENT);
	if (o)
	{
		o->manual = manual;
		o->signalled = initial;
	}
	return o;
}

DWORD WaitForSingleObject (HANDLE object, DWORD milliseconds)
{
	struct object *o = object;
	if (!o)
	{
		return WAIT_FAILED;
	}
	if (o->kind == OBJECT_MUTEX)
	{
		if (milliseconds == INFINITE)
		{
			return pthread_mutex_lock (&o->mutex) == 0 ? WAIT_OBJECT_0 : WAIT_FAILED;
		}
		struct timespec ts;
		clock_gettime (CLOCK_REALTIME, &ts);
		ts.tv_sec += milliseconds / 1000;
		ts.tv_nsec += (long) (milliseconds % 1000) * 1000000L;
		if (ts.tv_nsec >= 1000000000L)
		{
			ts.tv_sec++;
			ts.tv_nsec -= 1000000000L;
		}
		return pthread_mutex_timedlock (&o->mutex, &ts) == 0 ? WAIT_OBJECT_0 : WAIT_TIMEOUT;
	}
	if (o->kind == OBJECT_EVENT)
	{
		DWORD result = WAIT_OBJECT_0;
		pthread_mutex_lock (&o->mutex);
		if (!o->signalled)
		{
			if (milliseconds == 0)
			{
				result = WAIT_TIMEOUT;
			}
			else
			{
				struct timespec ts;
				clock_gettime (CLOCK_REALTIME, &ts);
				ts.tv_sec += milliseconds == INFINITE ? 1000000 : milliseconds / 1000;
				ts.tv_nsec += milliseconds == INFINITE ? 0 : (long) (milliseconds % 1000) * 1000000L;
				if (ts.tv_nsec >= 1000000000L)
				{
					ts.tv_sec++;
					ts.tv_nsec -= 1000000000L;
				}
				while (!o->signalled)
				{
					if (pthread_cond_timedwait (&o->cond, &o->mutex, &ts) != 0)
					{
						result = WAIT_TIMEOUT;
						break;
					}
				}
			}
		}
		if (result == WAIT_OBJECT_0 && !o->manual)
		{
			o->signalled = 0;
		}
		pthread_mutex_unlock (&o->mutex);
		return result;
	}
	return WAIT_FAILED;
}

void Sleep (DWORD milliseconds)
{
	struct timespec ts = { (time_t) (milliseconds / 1000), (long) (milliseconds % 1000) * 1000000L };
	nanosleep (&ts, NULL);
}

DWORD GetLastError (void)
{
	return last_error;
}

DWORD GetCurrentThreadId (void)
{
	return (DWORD) (uintptr_t) pthread_self ();
}

DWORD GetCurrentDirectory (DWORD size, LPSTR buffer)
{
	if (!getcwd (buffer, size))
	{
		return 0;
	}
	return (DWORD) strlen (buffer);
}

BOOL SetCurrentDirectory (LPCSTR path)
{
	char native[PATH_MAX];
	eech_native_path (path, native, sizeof (native));
	return chdir (native) == 0;
}

DWORD GetModuleFileName (HMODULE module, LPSTR filename, DWORD size)
{
	(void) module;
#ifdef _WIN32
	/* the host executable, as /proc/self/exe is on Linux */
	/* msvcrt.dll has _pgmptr (its _get_pgmptr is MSVC 8 on) */
	const char *exe = _pgmptr;
	if (!exe || !size)
	{
		return 0;
	}
	snprintf (filename, size, "%s", exe);
	return (DWORD) strlen (filename);
#else
	ssize_t n = readlink ("/proc/self/exe", filename, size ? size - 1 : 0);
	if (n < 0)
	{
		return 0;
	}
	filename[n] = 0;
	return (DWORD) n;
#endif
}

void GetSystemTime (LPSYSTEMTIME st)
{
	struct timeval tv;
	struct tm tm;
	gettimeofday (&tv, NULL);
#ifdef _WIN32
	time_t seconds = (time_t) tv.tv_sec;
	tm = *gmtime (&seconds);
#else
	gmtime_r (&tv.tv_sec, &tm);
#endif
	st->wYear = (WORD) (tm.tm_year + 1900);
	st->wMonth = (WORD) (tm.tm_mon + 1);
	st->wDayOfWeek = (WORD) tm.tm_wday;
	st->wDay = (WORD) tm.tm_mday;
	st->wHour = (WORD) tm.tm_hour;
	st->wMinute = (WORD) tm.tm_min;
	st->wSecond = (WORD) tm.tm_sec;
	st->wMilliseconds = (WORD) (tv.tv_usec / 1000);
}

HMODULE LoadLibrary (LPCSTR name)
{
	/* the only library EECH loads is the TrackIR client */
	(void) name;
	return NULL;
}

void *GetProcAddress (HMODULE module, LPCSTR name)
{
	(void) module;
	(void) name;
	return NULL;
}

BOOL FreeLibrary (HMODULE module)
{
	(void) module;
	return TRUE;
}

/* registry: no keys (EECH falls back to its defaults) */
LONG RegOpenKey (HKEY key, LPCSTR sub_key, HKEY *result)
{
	(void) key;
	(void) sub_key;
	*result = NULL;
	return 2;
}

LONG RegQueryValueEx (HKEY key, LPCSTR name, LPDWORD reserved, LPDWORD type, LPBYTE data, LPDWORD size)
{
	(void) key;
	(void) name;
	(void) reserved;
	(void) type;
	(void) data;
	(void) size;
	return 2;
}

LONG RegCloseKey (HKEY key)
{
	(void) key;
	return 0;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* winmm */

DWORD timeGetTime (void)
{
	return eech_headless_time_ms ();
}

MMRESULT timeGetDevCaps (LPTIMECAPS caps, UINT size)
{
	(void) size;
	caps->wPeriodMin = 1;
	caps->wPeriodMax = 1000000;
	return TIMERR_NOERROR;
}

MMRESULT timeBeginPeriod (UINT period)
{
	(void) period;
	return TIMERR_NOERROR;
}

MMRESULT timeEndPeriod (UINT period)
{
	(void) period;
	return TIMERR_NOERROR;
}

/* periodic multimedia timers are driven by the host's frame pump: none are started headless */
MMRESULT timeSetEvent (UINT delay, UINT resolution, LPTIMECALLBACK callback, DWORD_PTR user, UINT flags)
{
	(void) delay;
	(void) resolution;
	(void) callback;
	(void) user;
	(void) flags;
	return 0;
}

MMRESULT timeKillEvent (UINT id)
{
	(void) id;
	return TIMERR_NOERROR;
}

MCIERROR mciSendCommand (MCIDEVICEID device, UINT message, DWORD_PTR flags, DWORD_PTR parameters)
{
	(void) device;
	(void) message;
	(void) flags;
	(void) parameters;
	return 275; /* MCIERR_DEVICE_NOT_INSTALLED */
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* user32: no window */

UINT_PTR SetTimer (HWND window, UINT_PTR id, UINT elapse, TIMERPROC callback)
{
	(void) window;
	(void) elapse;
	(void) callback;
	return id;
}

BOOL KillTimer (HWND window, UINT_PTR id)
{
	(void) window;
	(void) id;
	return TRUE;
}

LRESULT SendMessage (HWND window, UINT message, WPARAM wparam, LPARAM lparam)
{
	(void) window;
	(void) message;
	(void) wparam;
	(void) lparam;
	return 0;
}

HWND SetFocus (HWND window)
{
	return window;
}

LONG SetWindowLong (HWND window, int index, LONG value)
{
	(void) window;
	(void) index;
	(void) value;
	return 0;
}

LONG GetWindowLong (HWND window, int index)
{
	(void) window;
	(void) index;
	return 0;
}

BOOL GetClientRect (HWND window, LPRECT rect)
{
	(void) window;
	rect->left = rect->top = 0;
	rect->right = 640;
	rect->bottom = 480;
	return TRUE;
}

BOOL GetCursorPos (LPPOINT point)
{
	point->x = point->y = 0;
	return TRUE;
}

BOOL ClientToScreen (HWND window, LPPOINT point)
{
	(void) window;
	(void) point;
	return TRUE;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* winsock */

int WSAStartup (WORD version, LPWSADATA data)
{
	memset (data, 0, sizeof (*data));
	data->wVersion = data->wHighVersion = version;
	return 0;
}

int WSACleanup (void)
{
	return 0;
}

int WSAGetLastError (void)
{
	return errno;
}

#ifdef _WIN32
/* the master-server heartbeat's sockets (compat/winsock.h): no network headless */
SOCKET socket (int family, int type, int protocol)
{
	(void) family;
	(void) type;
	(void) protocol;
	errno = EAFNOSUPPORT;
	return INVALID_SOCKET;
}

int sendto (SOCKET s, const char *data, int length, int flags, const struct sockaddr *to, int to_length)
{
	(void) s, (void) data, (void) length, (void) flags, (void) to, (void) to_length;
	return SOCKET_ERROR;
}

int recvfrom (SOCKET s, char *data, int length, int flags, struct sockaddr *from, int *from_length)
{
	(void) s, (void) data, (void) length, (void) flags, (void) from, (void) from_length;
	return SOCKET_ERROR;
}

int select (int n, fd_set *read, fd_set *write, fd_set *except, const struct timeval *timeout)
{
	(void) n, (void) read, (void) write, (void) except, (void) timeout;
	return 0;
}

struct hostent *gethostbyname (const char *name)
{
	(void) name;
	return NULL;
}

unsigned long inet_addr (const char *address)
{
	(void) address;
	return INADDR_NONE;
}

char *inet_ntoa (struct in_addr address)
{
	static char text[16];
	const uint8_t *b = (const uint8_t *) &address.s_addr;
	snprintf (text, sizeof (text), "%u.%u.%u.%u", b[0], b[1], b[2], b[3]);
	return text;
}

uint16_t htons (uint16_t value)
{
	return (uint16_t) ((value >> 8) | (value << 8));
}

int closesocket (SOCKET s)
{
	(void) s;
	return 0;
}
#endif

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* C file API */

FILE *eech_fopen_untraced (const char *name, const char *mode);

#undef fopen
#undef unlink

struct text_file
{
	char *data;
	size_t size, position;
};

static ssize_t text_read (void *cookie, char *buffer, size_t size)
{
	struct text_file *t = cookie;
	size_t n = t->size - t->position;
	if (n > size)
	{
		n = size;
	}
	memcpy (buffer, t->data + t->position, n);
	t->position += n;
	return (ssize_t) n;
}

static int text_seek (void *cookie, off64_t *offset, int whence)
{
	struct text_file *t = cookie;
	off64_t base = whence == SEEK_SET ? 0 : whence == SEEK_CUR ? (off64_t) t->position : (off64_t) t->size;
	off64_t p = base + *offset;
	if (p < 0 || p > (off64_t) t->size)
	{
		return -1;
	}
	t->position = (size_t) p;
	*offset = p;
	return 0;
}

static int text_close (void *cookie)
{
	struct text_file *t = cookie;
	free (t->data);
	free (t);
	return 0;
}

#ifdef _WIN32
/*
 * A read stream over bytes, without fmemopen or fopencookie: a binary
 * temporary file the CRT deletes when it is closed ("D"). Binary, so seeks
 * are exact (the CRT's text-mode ftell and fseek miscount on LF-only files).
 */
static FILE *memory_stream (const void *data, size_t size)
{
	static unsigned serial;
	const char *dir = getenv ("TEMP");
	char path[PATH_MAX];
	snprintf (path, sizeof (path), "%s/eech_%d_%u.tmp", dir ? dir : ".", (int) getpid (), serial++);
	FILE *f = fopen (path, "w+bD");
	if (f)
	{
		if (size && fwrite (data, 1, size, f) != size)
		{
			fclose (f);
			return NULL;
		}
		rewind (f);
	}
	return f;
}
#endif

/*
 * Windows text-mode read: CR LF becomes LF and Ctrl-Z ends the file. EECH's
 * tag parser depends on it (a CR after a float is not a terminator).
 */
static FILE *open_text_for_reading (const char *native)
{
	FILE *f = fopen (native, "rb");
	if (!f)
	{
		return NULL;
	}
	struct text_file *t = calloc (1, sizeof (*t));
	struct stat st;
	char *data = NULL;
	if (!t || fstat (fileno (f), &st) != 0 || !(data = malloc ((size_t) st.st_size + 1)))
	{
		free (t);
		fclose (f);
		return NULL;
	}
	size_t size = fread (data, 1, (size_t) st.st_size, f), n = 0;
	fclose (f);
	for (size_t i = 0; i < size; i++)
	{
		if (data[i] == 0x1a)
		{
			break;
		}
		if (data[i] == '\r' && i + 1 < size && data[i + 1] == '\n')
		{
			continue;
		}
		data[n++] = data[i];
	}
	t->data = data;
	t->size = n;
#ifdef _WIN32
	FILE *s = memory_stream (t->data, t->size);
	text_close (t);
	return s;
#else
	cookie_io_functions_t io = { text_read, NULL, text_seek, text_close };
	FILE *s = fopencookie (t, "r", io);
	if (!s)
	{
		text_close (t);
	}
	return s;
#endif
}

/*
 * Artwork substitution: the retail UI artwork (.psd screens and buttons) is
 * not part of the EECH source repository and nothing headless displays it.
 * A read of a missing .psd yields a valid 1x1 RGB Photoshop file, and each
 * substitution is logged.
 */
static const unsigned char placeholder_psd[] =
{
	'8', 'B', 'P', 'S', 0, 1, 0, 0, 0, 0, 0, 0,	/* signature, version 1, reserved */
	0, 3,						/* channels */
	0, 0, 0, 1, 0, 0, 0, 1,				/* height, width */
	0, 8, 0, 3,					/* depth 8, RGB */
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,		/* colour mode data, resources, layers: empty */
	0, 0,						/* raw image data */
	0x80, 0x80, 0x80				/* one grey pixel, planar R G B */
};

/* 1x1 24-bit Windows bitmap */
static const unsigned char placeholder_bmp[] =
{
	'B', 'M', 58, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0,			/* file header */
	40, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 24, 0,		/* info header: 1x1, 1 plane, 24 bits */
	0, 0, 0, 0, 4, 0, 0, 0, 0x13, 0x0b, 0, 0, 0x13, 0x0b, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0,
	0x80, 0x80, 0x80, 0						/* one grey pixel, row padded to 4 bytes */
};

/* 1x1 uncompressed 24-bit Targa */
static const unsigned char placeholder_tga[] =
{
	0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 24, 0,	/* header: true colour, 1x1, 24 bits */
	0x80, 0x80, 0x80					/* one grey pixel, BGR */
};

static FILE *placeholder_artwork (const char *name, const char *native)
{
	size_t n = strlen (native);
	const void *data = NULL;
	size_t size = 0;
	if (n >= 4 && strcasecmp (native + n - 4, ".psd") == 0)
	{
		data = placeholder_psd;
		size = sizeof (placeholder_psd);
	}
	else if (n >= 4 && strcasecmp (native + n - 4, ".bmp") == 0)
	{
		data = placeholder_bmp;
		size = sizeof (placeholder_bmp);
	}
	else if (n >= 4 && strcasecmp (native + n - 4, ".tga") == 0)
	{
		data = placeholder_tga;
		size = sizeof (placeholder_tga);
	}
	else
	{
		return NULL;
	}
	eech_log (1, "artwork not installed, substituting a 1x1 image: %s", name);
#ifdef _WIN32
	return memory_stream (data, size);
#else
	return fmemopen ((void *) data, size, "rb");
#endif
}

/* EECH_TRACE_FILES=1: log every file the engine opens (and every miss) */
static int trace_files = -1;

static void trace_open (const char *native, const char *mode, const void *result)
{
	if (trace_files < 0)
	{
		trace_files = getenv ("EECH_TRACE_FILES") != NULL;
	}
	if (trace_files)
	{
		eech_log (2, "open %s %s: %s", mode, native, result ? "ok" : "missing");
	}
}

FILE *eech_fopen (const char *name, const char *mode)
{
	FILE *f = eech_fopen_untraced (name, mode);
	char native[PATH_MAX];
	eech_native_path (name, native, sizeof (native));
	trace_open (native, mode, f);
	return f;
}

FILE *eech_fopen_untraced (const char *name, const char *mode)
{
	char native[PATH_MAX];
	eech_native_path (name, native, sizeof (native));
	if (mode[0] == 'r' && access (native, R_OK) != 0)
	{
		/* safe_fopen retries under cohokum\ and common\: substitute only on the last attempt */
		const char *tail = strstr (name, "\\common\\");
		if (tail)
		{
			return placeholder_artwork (name, native);
		}
		return NULL;
	}
	if (mode[0] == 'r' && !strchr (mode, 'b') && !strchr (mode, '+'))
	{
		return open_text_for_reading (native);
	}
	char m[8];
	size_t j = 0;
	for (size_t i = 0; mode[i] && j + 1 < sizeof (m); i++)
	{
		if (mode[i] != 't')
		{
			m[j++] = mode[i];
		}
	}
#ifdef _WIN32
	/* binary, as every mode is on Linux: the CRT's text mode would write CR LF */
	if (!strchr (m, 'b') && j + 1 < sizeof (m))
	{
		m[j++] = 'b';
	}
#endif
	m[j] = 0;
	return fopen (native, m);
}

int eech_unlink (const char *name)
{
	char native[PATH_MAX];
	eech_native_path (name, native, sizeof (native));
	return unlink (native);
}
