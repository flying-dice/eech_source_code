/*
 * EECH headless build: the Win32 and MSVC CRT functions the EECH sources
 * call, declared so that pointer results are not truncated through implicit
 * int declarations. csrc/eech_win32.c implements them over POSIX.
 */
#ifndef EECH_COMPAT_API_H
#define EECH_COMPAT_API_H

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <strings.h>

#define ARRAYSIZE(a) (sizeof (a) / sizeof ((a)[0]))
#define MCI_MSF_MINUTE(msf) ((BYTE) (msf))
#define MCI_MSF_SECOND(msf) ((BYTE) (((WORD) (msf)) >> 8))
#define MCI_MSF_FRAME(msf) ((BYTE) ((msf) >> 16))
#define GetWindowStyle(hwnd) ((DWORD) GetWindowLong (hwnd, GWL_STYLE))

/* MSVC CRT */
#define stricmp strcasecmp
#define strcmpi strcasecmp
#define strnicmp strncasecmp
#define _stricmp strcasecmp
#define _strnicmp strncasecmp
#define _snprintf snprintf
#define _vsnprintf vsnprintf
char *strupr (char *s);
char *strlwr (char *s);
char *itoa (int value, char *buffer, int radix);

/* C file API: Windows paths and text mode (see csrc/eech_win32.c) */
FILE *eech_fopen (const char *name, const char *mode);
int eech_unlink (const char *name);
#define fopen eech_fopen
#define unlink eech_unlink

/* kernel */
void Sleep (DWORD milliseconds);
DWORD GetLastError (void);
DWORD GetCurrentThreadId (void);
DWORD GetCurrentDirectory (DWORD size, LPSTR buffer);
BOOL SetCurrentDirectory (LPCSTR path);
DWORD GetModuleFileName (HMODULE module, LPSTR filename, DWORD size);
void GetSystemTime (LPSYSTEMTIME time);
BOOL CloseHandle (HANDLE object);
HANDLE CreateFile (LPCSTR name, DWORD access, DWORD share, LPSECURITY_ATTRIBUTES security, DWORD disposition, DWORD flags, HANDLE template_file);
BOOL ReadFile (HANDLE file, LPVOID buffer, DWORD size, LPDWORD read, LPOVERLAPPED overlapped);
DWORD GetFileSize (HANDLE file, LPDWORD high);
HANDLE CreateFileMapping (HANDLE file, LPSECURITY_ATTRIBUTES security, DWORD protect, DWORD size_high, DWORD size_low, LPCSTR name);
LPVOID MapViewOfFile (HANDLE mapping, DWORD access, DWORD offset_high, DWORD offset_low, SIZE_T size);
BOOL UnmapViewOfFile (LPCVOID base);
LPVOID VirtualAlloc (LPVOID address, SIZE_T size, DWORD type, DWORD protect);
BOOL VirtualFree (LPVOID address, SIZE_T size, DWORD type);
HANDLE FindFirstFile (LPCSTR pattern, WIN32_FIND_DATA *data);
BOOL FindNextFile (HANDLE find, WIN32_FIND_DATA *data);
BOOL FindClose (HANDLE find);
HANDLE CreateMutex (LPSECURITY_ATTRIBUTES security, BOOL owner, LPCSTR name);
BOOL ReleaseMutex (HANDLE mutex);
HANDLE CreateEvent (LPSECURITY_ATTRIBUTES security, BOOL manual, BOOL initial, LPCSTR name);
DWORD WaitForSingleObject (HANDLE object, DWORD milliseconds);
HMODULE LoadLibrary (LPCSTR name);
void *GetProcAddress (HMODULE module, LPCSTR name);
BOOL FreeLibrary (HMODULE module);
LONG RegOpenKey (HKEY key, LPCSTR sub_key, HKEY *result);
LONG RegQueryValueEx (HKEY key, LPCSTR name, LPDWORD reserved, LPDWORD type, LPBYTE data, LPDWORD size);
LONG RegCloseKey (HKEY key);

/* winmm */
DWORD timeGetTime (void);
MMRESULT timeGetDevCaps (LPTIMECAPS caps, UINT size);
MMRESULT timeBeginPeriod (UINT period);
MMRESULT timeEndPeriod (UINT period);
MMRESULT timeSetEvent (UINT delay, UINT resolution, LPTIMECALLBACK callback, DWORD_PTR user, UINT flags);
MMRESULT timeKillEvent (UINT id);
MCIERROR mciSendCommand (MCIDEVICEID device, UINT message, DWORD_PTR flags, DWORD_PTR parameters);

/* user32: there is no window */
UINT_PTR SetTimer (HWND window, UINT_PTR id, UINT elapse, TIMERPROC callback);
BOOL KillTimer (HWND window, UINT_PTR id);
LRESULT SendMessage (HWND window, UINT message, WPARAM wparam, LPARAM lparam);
HWND SetFocus (HWND window);
LONG SetWindowLong (HWND window, int index, LONG value);
LONG GetWindowLong (HWND window, int index);
BOOL GetClientRect (HWND window, LPRECT rect);
BOOL GetCursorPos (LPPOINT point);
BOOL ClientToScreen (HWND window, LPPOINT point);

/* aphavoc/source/cdmusic.c, called from modules/sound/soundman.c without a declaration */
void pause_music (void);
void unpause_music (void);

#endif
