/*
 * EECH headless build, Windows targets only (force-included by build/main.rs):
 * the Win32 and CRT functions csrc/ implements for EECH are renamed eech_w32_*.
 *
 * On Linux csrc/ is the only implementation of these names. On Windows the
 * system DLLs export the same names (CloseHandle, WaitForSingleObject, Sleep,
 * GetLastError, ...), and the Rust standard library and the C runtime call
 * them: were EECH's emulations linked under the real names, they would take
 * those calls over. Renamed, EECH still calls the same emulation it runs on
 * Linux (so a Windows run matches a Linux one), and everything else in the
 * process keeps the real API.
 */

#ifndef EECH_WIN32_NAMES_H
#define EECH_WIN32_NAMES_H

#ifdef _WIN32

/* kernel32 */
#define CloseHandle eech_w32_CloseHandle
#define CreateEvent eech_w32_CreateEvent
#define CreateFile eech_w32_CreateFile
#define CreateFileMapping eech_w32_CreateFileMapping
#define CreateMutex eech_w32_CreateMutex
#define FindClose eech_w32_FindClose
#define FindFirstFile eech_w32_FindFirstFile
#define FindNextFile eech_w32_FindNextFile
#define FreeLibrary eech_w32_FreeLibrary
#define GetCurrentDirectory eech_w32_GetCurrentDirectory
#define GetCurrentThreadId eech_w32_GetCurrentThreadId
#define GetFileSize eech_w32_GetFileSize
#define GetLastError eech_w32_GetLastError
#define GetModuleFileName eech_w32_GetModuleFileName
#define GetProcAddress eech_w32_GetProcAddress
#define GetSystemTime eech_w32_GetSystemTime
#define LoadLibrary eech_w32_LoadLibrary
#define MapViewOfFile eech_w32_MapViewOfFile
#define ReadFile eech_w32_ReadFile
#define ReleaseMutex eech_w32_ReleaseMutex
#define SetCurrentDirectory eech_w32_SetCurrentDirectory
#define Sleep eech_w32_Sleep
#define UnmapViewOfFile eech_w32_UnmapViewOfFile
#define VirtualAlloc eech_w32_VirtualAlloc
#define VirtualFree eech_w32_VirtualFree
#define WaitForSingleObject eech_w32_WaitForSingleObject

/* user32 */
#define ClientToScreen eech_w32_ClientToScreen
#define GetClientRect eech_w32_GetClientRect
#define GetCursorPos eech_w32_GetCursorPos
#define GetWindowLong eech_w32_GetWindowLong
#define KillTimer eech_w32_KillTimer
#define SendMessage eech_w32_SendMessage
#define SetFocus eech_w32_SetFocus
#define SetTimer eech_w32_SetTimer
#define SetWindowLong eech_w32_SetWindowLong

/* advapi32 */
#define RegCloseKey eech_w32_RegCloseKey
#define RegOpenKey eech_w32_RegOpenKey
#define RegQueryValueEx eech_w32_RegQueryValueEx

/* ws2_32, winmm, DirectX */
#define WSACleanup eech_w32_WSACleanup
#define WSAGetLastError eech_w32_WSAGetLastError
#define WSAStartup eech_w32_WSAStartup
#define closesocket eech_w32_closesocket
#define gethostbyname eech_w32_gethostbyname
#define htons eech_w32_htons
#define inet_addr eech_w32_inet_addr
#define inet_ntoa eech_w32_inet_ntoa
#define recvfrom eech_w32_recvfrom
#define select eech_w32_select
#define sendto eech_w32_sendto
#define socket eech_w32_socket
#define mciSendCommand eech_w32_mciSendCommand
#define timeBeginPeriod eech_w32_timeBeginPeriod
#define timeEndPeriod eech_w32_timeEndPeriod
#define timeGetDevCaps eech_w32_timeGetDevCaps
#define timeGetTime eech_w32_timeGetTime
#define timeKillEvent eech_w32_timeKillEvent
#define timeSetEvent eech_w32_timeSetEvent
#define DirectInputCreateEx eech_w32_DirectInputCreateEx
#define DirectSoundCreate eech_w32_DirectSoundCreate

/* the MSVC CRT */
#define _findclose eech_w32__findclose
#define _findfirst eech_w32__findfirst
#define _findnext eech_w32__findnext
#define itoa eech_w32_itoa
#define strlwr eech_w32_strlwr
#define strupr eech_w32_strupr

#endif

#endif
