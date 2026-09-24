/* EECH headless build: <winsock.h> over the POSIX socket API */
#ifndef EECH_COMPAT_WINSOCK_H
#define EECH_COMPAT_WINSOCK_H
#include "windows.h"
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <netdb.h>
#include <unistd.h>

typedef int SOCKET;
typedef struct sockaddr SOCKADDR, *LPSOCKADDR;
typedef struct sockaddr_in SOCKADDR_IN, *LPSOCKADDR_IN;
typedef struct hostent HOSTENT, *LPHOSTENT;
typedef struct in_addr IN_ADDR;
typedef struct WSAData { WORD wVersion, wHighVersion; char szDescription[257], szSystemStatus[129]; unsigned short iMaxSockets, iMaxUdpDg;
	char *lpVendorInfo; } WSADATA, *LPWSADATA;
#define INVALID_SOCKET (-1)
#define SOCKET_ERROR (-1)
#define closesocket close
int WSAStartup (WORD version, LPWSADATA data);
int WSACleanup (void);
int WSAGetLastError (void);
#endif
