/*
 * EECH headless build: the part of the Windows SDK and DirectX surface the
 * EECH sources name, declared for a non-Windows compiler. Types follow the
 * Win64 ABI (LLP64: LONG and DWORD are 32-bit, handles and W/LPARAM are
 * pointer-sized). Nothing here renders, plays or reads input: the headless
 * platform layer (csrc/) implements the functions the simulation reaches and
 * fails loudly on the rest.
 */
#ifndef EECH_COMPAT_WINDOWS_H
#define EECH_COMPAT_WINDOWS_H

#include <stdint.h>
#include <stddef.h>
#include <string.h>
#include <strings.h>

typedef int BOOL;
typedef unsigned char BYTE;
typedef unsigned short WORD;
typedef uint32_t DWORD;
typedef int32_t LONG;
typedef uint32_t ULONG;
typedef int16_t SHORT;
typedef uint16_t USHORT;
typedef unsigned int UINT;
typedef int INT;
typedef char CHAR;
typedef unsigned char UCHAR;
typedef float FLOAT;
typedef double DOUBLE;
typedef void VOID;
typedef void *PVOID, *LPVOID;
typedef const void *LPCVOID;
typedef char *LPSTR, *PSTR;
typedef const char *LPCSTR, *PCSTR;
typedef char TCHAR;
typedef LPSTR LPTSTR;
typedef LPCSTR LPCTSTR;
typedef uint16_t WCHAR;
typedef WCHAR *LPWSTR;
typedef const WCHAR *LPCWSTR;
typedef int64_t LONGLONG;
#ifndef __MINGW32__
/* MinGW-w64 predefines __int64 (long long) */
typedef int64_t __int64;
#endif
typedef uint64_t ULONGLONG, DWORDLONG;
typedef uintptr_t WPARAM, UINT_PTR, ULONG_PTR, DWORD_PTR, SIZE_T;
typedef intptr_t LPARAM, LRESULT, INT_PTR, LONG_PTR;
typedef DWORD *LPDWORD, *PDWORD;
typedef LONG *LPLONG;
typedef BYTE *LPBYTE;
typedef WORD *LPWORD;
typedef BOOL *LPBOOL;
/*
 * 64-bit, as long is on Linux (LP64), where the platform layer was developed:
 * there E_FAIL (0x80004005) is positive, so FAILED () is false for the null
 * COM stubs' E_FAIL, and EECH's Direct3D set-up takes its success path, which
 * a headless run needs. A 32-bit HRESULT (Windows' long) would make it fatal.
 */
typedef int64_t HRESULT;
typedef WORD ATOM;
typedef DWORD COLORREF;

#define DECLARE_EECH_HANDLE(NAME) typedef struct NAME##__ { int unused; } *NAME
DECLARE_EECH_HANDLE (HWND);
DECLARE_EECH_HANDLE (HINSTANCE);
DECLARE_EECH_HANDLE (HDC);
DECLARE_EECH_HANDLE (HMENU);
DECLARE_EECH_HANDLE (HICON);
DECLARE_EECH_HANDLE (HBRUSH);
DECLARE_EECH_HANDLE (HBITMAP);
DECLARE_EECH_HANDLE (HFONT);
DECLARE_EECH_HANDLE (HKEY);
DECLARE_EECH_HANDLE (HGLRC);
DECLARE_EECH_HANDLE (HMONITOR);
DECLARE_EECH_HANDLE (HPALETTE);
DECLARE_EECH_HANDLE (HGDIOBJ);
DECLARE_EECH_HANDLE (HRGN);
typedef HICON HCURSOR;
typedef HINSTANCE HMODULE;
typedef void *HANDLE;
typedef HANDLE *LPHANDLE;

typedef struct _GUID { uint32_t Data1; uint16_t Data2; uint16_t Data3; uint8_t Data4[8]; } GUID, *LPGUID;
typedef const GUID *REFGUID, *LPCGUID;
typedef GUID IID, CLSID;
typedef const GUID *REFIID, *REFCLSID;

typedef struct _RTL_CRITICAL_SECTION { void *opaque; } CRITICAL_SECTION, *LPCRITICAL_SECTION;

typedef struct tagPOINT { LONG x, y; } POINT, *LPPOINT;
typedef struct tagRECT { LONG left, top, right, bottom; } RECT, *LPRECT;
typedef const RECT *LPCRECT;
typedef struct tagSIZE { LONG cx, cy; } SIZE, *LPSIZE;
typedef struct tagMSG { HWND hwnd; UINT message; WPARAM wParam; LPARAM lParam; DWORD time; POINT pt; } MSG, *LPMSG;
typedef union _LARGE_INTEGER { struct { DWORD LowPart; LONG HighPart; } u; LONGLONG QuadPart; } LARGE_INTEGER;
typedef struct _FILETIME { DWORD dwLowDateTime, dwHighDateTime; } FILETIME;
typedef struct _SYSTEMTIME { WORD wYear, wMonth, wDayOfWeek, wDay, wHour, wMinute, wSecond, wMilliseconds; } SYSTEMTIME, *LPSYSTEMTIME;

#define TRUE 1
#define FALSE 0
#define WINAPI
#define CALLBACK
#define APIENTRY
#define PASCAL
#define FAR
#define NEAR
#define far
#define near
#define __cdecl
#define __stdcall
#define __fastcall
#define _cdecl
#define __forceinline inline
#define CONST const
#define IN
#define OUT
#define MAX_PATH 260
#define INFINITE 0xFFFFFFFFu
#define S_OK ((HRESULT) 0)
#define S_FALSE ((HRESULT) 1)
#define E_FAIL ((HRESULT) 0x80004005L)
#define SUCCEEDED(hr) (((HRESULT) (hr)) >= 0)
#define FAILED(hr) (((HRESULT) (hr)) < 0)
#define MAKEWORD(a, b) ((WORD) (((BYTE) (a)) | ((WORD) ((BYTE) (b))) << 8))
#define MAKELONG(a, b) ((LONG) (((WORD) (a)) | ((DWORD) ((WORD) (b))) << 16))
#define LOWORD(l) ((WORD) ((DWORD_PTR) (l) & 0xffff))
#define HIWORD(l) ((WORD) (((DWORD_PTR) (l) >> 16) & 0xffff))
#define LOBYTE(w) ((BYTE) ((DWORD_PTR) (w) & 0xff))
#define HIBYTE(w) ((BYTE) (((DWORD_PTR) (w) >> 8) & 0xff))
#define RGB(r, g, b) ((COLORREF) (((BYTE) (r) | ((WORD) ((BYTE) (g)) << 8)) | (((DWORD) (BYTE) (b)) << 16)))
#define ZeroMemory(p, n) memset ((p), 0, (n))
#define CopyMemory(d, s, n) memcpy ((d), (s), (n))
#define FillMemory(p, n, v) memset ((p), (v), (n))

#include "eech_dx.h"
#include "eech_api.h"
#include "eech_com.h"

#endif
