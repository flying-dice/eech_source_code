/* EECH headless build: <winsock.h> over the POSIX socket API (Linux), or as failing stubs (Windows) */
#ifndef EECH_COMPAT_WINSOCK_H
#define EECH_COMPAT_WINSOCK_H
#include "windows.h"
#include <sys/types.h>

#ifdef __MINGW32__
/*
 * MinGW-w64: the real Winsock headers need the real <windows.h>, which the
 * compat headers stand in for. EECH's only socket use is the master-server
 * heartbeat (comms/comm_man.c), which a headless campaign has no network for:
 * the calls it makes are declared here and fail (csrc/eech_win32.c), under
 * eech_w32_* names (eech_win32_names.h).
 */
#include <sys/time.h>

typedef uintptr_t SOCKET;
#ifndef _BSDTYPES_DEFINED
/* as MinGW-w64's <_bsd_types.h> */
typedef unsigned long u_long;
#endif
struct in_addr { uint32_t s_addr; };
struct sockaddr { uint16_t sa_family; char sa_data[14]; };
struct sockaddr_in { int16_t sin_family; uint16_t sin_port; struct in_addr sin_addr; char sin_zero[8]; };
struct hostent { char *h_name; char **h_aliases; short h_addrtype; short h_length; char **h_addr_list; };
typedef struct { unsigned int fd_count; SOCKET fd_array[64]; } fd_set;
#define FD_ZERO(set) ((set)->fd_count = 0)
#define FD_SET(fd, set) ((set)->fd_count < 64 ? (void) ((set)->fd_array[(set)->fd_count++] = (SOCKET) (fd)) : (void) 0)
#define FD_ISSET(fd, set) 0
#define AF_INET 2
#define SOCK_DGRAM 2
#define IPPROTO_UDP 17
#define INADDR_NONE 0xffffffffu
#define INVALID_SOCKET (~(SOCKET) 0)
#define SOCKET_ERROR (-1)

SOCKET socket (int family, int type, int protocol);
int sendto (SOCKET s, const char *data, int length, int flags, const struct sockaddr *to, int to_length);
int recvfrom (SOCKET s, char *data, int length, int flags, struct sockaddr *from, int *from_length);
int select (int n, fd_set *read, fd_set *write, fd_set *except, const struct timeval *timeout);
struct hostent *gethostbyname (const char *name);
unsigned long inet_addr (const char *address);
char *inet_ntoa (struct in_addr address);
uint16_t htons (uint16_t value);
int closesocket (SOCKET s);
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <unistd.h>

typedef int SOCKET;
#define INVALID_SOCKET (-1)
#define SOCKET_ERROR (-1)
#define closesocket close
#endif

typedef struct sockaddr SOCKADDR, *LPSOCKADDR;
typedef struct sockaddr_in SOCKADDR_IN, *LPSOCKADDR_IN;
typedef struct hostent HOSTENT, *LPHOSTENT;
typedef struct in_addr IN_ADDR;
typedef struct WSAData { WORD wVersion, wHighVersion; char szDescription[257], szSystemStatus[129]; unsigned short iMaxSockets, iMaxUdpDg;
	char *lpVendorInfo; } WSADATA, *LPWSADATA;
int WSAStartup (WORD version, LPWSADATA data);
int WSACleanup (void);
int WSAGetLastError (void);
#endif
