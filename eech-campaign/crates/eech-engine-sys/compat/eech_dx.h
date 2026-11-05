/*
 * EECH headless build: the DirectX and remaining Win32 names the EECH headers
 * declare with. COM interfaces are opaque: nothing headless calls through
 * them, and the backend sources that do are replaced by csrc/.
 */
#ifndef EECH_COMPAT_DX_H
#define EECH_COMPAT_DX_H

#include <time.h>
#include <errno.h>

typedef int WINBOOL;
typedef uint64_t UINT64;
typedef int64_t INT64;
typedef uint32_t UINT32;
typedef int32_t INT32;
typedef uint16_t UINT16;
typedef uint8_t UINT8;
typedef LONG *PLONG;
typedef UINT MMRESULT;
typedef DWORD MCIERROR;
typedef UINT MCIDEVICEID;

#define EECH_OPAQUE(I) typedef struct I I; typedef I *LP##I, *P##I
#define EECH_OPAQUE2(I, LP) typedef struct I I; typedef I *LP
EECH_OPAQUE2 (IDirect3D9, LPDIRECT3D9);
EECH_OPAQUE2 (IDirect3DDevice9, LPDIRECT3DDEVICE9);
EECH_OPAQUE2 (IDirect3DVertexBuffer9, LPDIRECT3DVERTEXBUFFER9);
EECH_OPAQUE2 (IDirect3DIndexBuffer9, LPDIRECT3DINDEXBUFFER9);
EECH_OPAQUE2 (IDirect3DTexture9, LPDIRECT3DTEXTURE9);
EECH_OPAQUE2 (IDirect3DSurface9, LPDIRECT3DSURFACE9);
EECH_OPAQUE2 (IDirect3DBaseTexture9, LPDIRECT3DBASETEXTURE9);
EECH_OPAQUE2 (IDirect3DCubeTexture9, LPDIRECT3DCUBETEXTURE9);
EECH_OPAQUE2 (IDirect3DStateBlock9, LPDIRECT3DSTATEBLOCK9);
EECH_OPAQUE2 (IDirect3DVertexDeclaration9, LPDIRECT3DVERTEXDECLARATION9);
EECH_OPAQUE2 (IDirect3DVertexShader9, LPDIRECT3DVERTEXSHADER9);
EECH_OPAQUE2 (IDirect3DPixelShader9, LPDIRECT3DPIXELSHADER9);
EECH_OPAQUE2 (IDirect3DQuery9, LPDIRECT3DQUERY9);
EECH_OPAQUE2 (ID3DXFont, LPD3DXFONT);
EECH_OPAQUE2 (ID3DXSprite, LPD3DXSPRITE);
EECH_OPAQUE2 (ID3DXBuffer, LPD3DXBUFFER);
EECH_OPAQUE2 (IDirectInput7, LPDIRECTINPUT7);
EECH_OPAQUE2 (IDirectInput, LPDIRECTINPUT);
EECH_OPAQUE2 (IDirectInputDevice7, LPDIRECTINPUTDEVICE7);
EECH_OPAQUE2 (IDirectInputDevice2, LPDIRECTINPUTDEVICE2);
EECH_OPAQUE2 (IDirectInputDevice, LPDIRECTINPUTDEVICE);
EECH_OPAQUE2 (IDirectInputEffect, LPDIRECTINPUTEFFECT);
EECH_OPAQUE2 (IDirectSound, LPDIRECTSOUND);
EECH_OPAQUE2 (IDirectSoundBuffer, LPDIRECTSOUNDBUFFER);
EECH_OPAQUE2 (IDirectSound3DBuffer, LPDIRECTSOUND3DBUFFER);
EECH_OPAQUE2 (IDirectSound3DListener, LPDIRECTSOUND3DLISTENER);
EECH_OPAQUE2 (IDirectPlay4, LPDIRECTPLAY4A);
EECH_OPAQUE2 (IDirectPlay2, LPDIRECTPLAY2A);
EECH_OPAQUE2 (IDirectPlayLobby3, LPDIRECTPLAYLOBBY3A);
EECH_OPAQUE2 (IDirectPlayLobby2, LPDIRECTPLAYLOBBY2A);
EECH_OPAQUE2 (IDirectPlayLobby, LPDIRECTPLAYLOBBYA);
EECH_OPAQUE2 (IDirectDraw7, LPDIRECTDRAWX);
EECH_OPAQUE2 (IDirectDrawSurface7, LPDIRECTDRAWSURFACEX);
EECH_OPAQUE2 (IDirectDrawClipper, LPDIRECTDRAWCLIPPER);
EECH_OPAQUE2 (IDirectDrawPalette, LPDIRECTDRAWPALETTE);
EECH_OPAQUE2 (IAMMultiMediaStream, LPAMMULTIMEDIASTREAM);
EECH_OPAQUE2 (IMediaStream, LPMEDIASTREAM);
EECH_OPAQUE2 (IUnknown, LPUNKNOWN);
typedef LPDIRECTPLAY4A LPDIRECTPLAYX;
typedef LPDIRECTPLAYLOBBY3A LPDIRECTPLAYLOBBYX;

/* Direct3D 9 value types */
typedef DWORD D3DCOLOR;
typedef enum { D3DFMT_UNKNOWN = 0, D3DFMT_R8G8B8 = 20, D3DFMT_A8R8G8B8 = 21, D3DFMT_X8R8G8B8 = 22, D3DFMT_R5G6B5 = 23, D3DFMT_X1R5G5B5 = 24,
	D3DFMT_A1R5G5B5 = 25, D3DFMT_A4R4G4B4 = 26, D3DFMT_A8 = 28, D3DFMT_D16 = 80, D3DFMT_D24S8 = 75, D3DFMT_D24X8 = 77, D3DFMT_D32 = 71,
	D3DFMT_INDEX16 = 101, D3DFMT_INDEX32 = 102, D3DFMT_FORCE_DWORD = 0x7fffffff } D3DFORMAT;
typedef enum { D3DMULTISAMPLE_NONE = 0, D3DMULTISAMPLE_FORCE_DWORD = 0x7fffffff } D3DMULTISAMPLE_TYPE;
typedef enum { D3DSWAPEFFECT_DISCARD = 1, D3DSWAPEFFECT_FLIP = 2, D3DSWAPEFFECT_COPY = 3, D3DSWAPEFFECT_FORCE_DWORD = 0x7fffffff } D3DSWAPEFFECT;
typedef enum { D3DDEVTYPE_HAL = 1, D3DDEVTYPE_REF = 2, D3DDEVTYPE_SW = 3, D3DDEVTYPE_FORCE_DWORD = 0x7fffffff } D3DDEVTYPE;
typedef enum { D3DPOOL_DEFAULT = 0, D3DPOOL_MANAGED = 1, D3DPOOL_SYSTEMMEM = 2, D3DPOOL_SCRATCH = 3, D3DPOOL_FORCE_DWORD = 0x7fffffff } D3DPOOL;
typedef enum { D3DRS_FORCE_DWORD = 0x7fffffff } D3DRENDERSTATETYPE;
typedef enum { D3DPT_POINTLIST = 1, D3DPT_LINELIST = 2, D3DPT_LINESTRIP = 3, D3DPT_TRIANGLELIST = 4, D3DPT_TRIANGLESTRIP = 5, D3DPT_TRIANGLEFAN = 6,
	D3DPT_FORCE_DWORD = 0x7fffffff } D3DPRIMITIVETYPE;
typedef enum { D3DTEXF_NONE = 0, D3DTEXF_POINT = 1, D3DTEXF_LINEAR = 2, D3DTEXF_ANISOTROPIC = 3, D3DTEXF_FORCE_DWORD = 0x7fffffff } D3DTEXTUREFILTERTYPE;
typedef enum { D3DBLEND_ZERO = 1, D3DBLEND_ONE = 2, D3DBLEND_SRCCOLOR = 3, D3DBLEND_INVSRCCOLOR = 4, D3DBLEND_SRCALPHA = 5, D3DBLEND_INVSRCALPHA = 6,
	D3DBLEND_DESTALPHA = 7, D3DBLEND_INVDESTALPHA = 8, D3DBLEND_DESTCOLOR = 9, D3DBLEND_INVDESTCOLOR = 10, D3DBLEND_FORCE_DWORD = 0x7fffffff } D3DBLEND;
typedef enum { D3DCMP_NEVER = 1, D3DCMP_LESS, D3DCMP_EQUAL, D3DCMP_LESSEQUAL, D3DCMP_GREATER, D3DCMP_NOTEQUAL, D3DCMP_GREATEREQUAL, D3DCMP_ALWAYS,
	D3DCMP_FORCE_DWORD = 0x7fffffff } D3DCMPFUNC;
typedef enum { D3DTS_VIEW = 2, D3DTS_PROJECTION = 3, D3DTS_WORLD = 256, D3DTS_FORCE_DWORD = 0x7fffffff } D3DTRANSFORMSTATETYPE;
typedef struct _D3DMATRIX { float m[4][4]; } D3DMATRIX, *LPD3DMATRIX;
typedef struct _D3DVECTOR { float x, y, z; } D3DVECTOR, *LPD3DVECTOR;
typedef struct _D3DCOLORVALUE { float r, g, b, a; } D3DCOLORVALUE;
typedef struct _D3DVIEWPORT9 { DWORD X, Y, Width, Height; float MinZ, MaxZ; } D3DVIEWPORT9;
typedef struct _D3DDISPLAYMODE { UINT Width, Height, RefreshRate; D3DFORMAT Format; } D3DDISPLAYMODE;
typedef struct _D3DPRESENT_PARAMETERS_ {
	UINT BackBufferWidth, BackBufferHeight; D3DFORMAT BackBufferFormat; UINT BackBufferCount;
	D3DMULTISAMPLE_TYPE MultiSampleType; DWORD MultiSampleQuality; D3DSWAPEFFECT SwapEffect; HWND hDeviceWindow; BOOL Windowed;
	BOOL EnableAutoDepthStencil; D3DFORMAT AutoDepthStencilFormat; DWORD Flags; UINT FullScreen_RefreshRateInHz, PresentationInterval;
} D3DPRESENT_PARAMETERS;
typedef struct _D3DCAPS9 {
	D3DDEVTYPE DeviceType; UINT AdapterOrdinal; DWORD Caps, Caps2, Caps3, PresentationIntervals, CursorCaps, DevCaps, PrimitiveMiscCaps, RasterCaps,
	ZCmpCaps, SrcBlendCaps, DestBlendCaps, AlphaCmpCaps, ShadeCaps, TextureCaps, TextureFilterCaps, CubeTextureFilterCaps, VolumeTextureFilterCaps,
	TextureAddressCaps, VolumeTextureAddressCaps, LineCaps, MaxTextureWidth, MaxTextureHeight, MaxVolumeExtent, MaxTextureRepeat, MaxTextureAspectRatio,
	MaxAnisotropy; float MaxVertexW, GuardBandLeft, GuardBandTop, GuardBandRight, GuardBandBottom, ExtentsAdjust; DWORD StencilCaps, FVFCaps,
	TextureOpCaps, MaxTextureBlendStages, MaxSimultaneousTextures, VertexProcessingCaps, MaxActiveLights, MaxUserClipPlanes, MaxVertexBlendMatrices,
	MaxVertexBlendMatrixIndex; float MaxPointSize; DWORD MaxPrimitiveCount, MaxVertexIndex, MaxStreams, MaxStreamStride, VertexShaderVersion,
	MaxVertexShaderConst, PixelShaderVersion; float PixelShader1xMaxValue; DWORD reserved[32];
} D3DCAPS9;
typedef struct _D3DLOCKED_RECT { INT Pitch; void *pBits; } D3DLOCKED_RECT;
typedef struct _D3DSURFACE_DESC { D3DFORMAT Format; DWORD Type, Usage; D3DPOOL Pool; D3DMULTISAMPLE_TYPE MultiSampleType; DWORD MultiSampleQuality;
	UINT Width, Height; } D3DSURFACE_DESC;
typedef struct _D3DLIGHT9 { DWORD Type; D3DCOLORVALUE Diffuse, Specular, Ambient; D3DVECTOR Position, Direction; float Range, Falloff, Attenuation0,
	Attenuation1, Attenuation2, Theta, Phi; } D3DLIGHT9;
typedef struct _D3DMATERIAL9 { D3DCOLORVALUE Diffuse, Ambient, Specular, Emissive; float Power; } D3DMATERIAL9;
#define D3D_OK S_OK
#define D3DCOLOR_ARGB(a, r, g, b) ((D3DCOLOR) ((((a) & 0xff) << 24) | (((r) & 0xff) << 16) | (((g) & 0xff) << 8) | ((b) & 0xff)))
#define D3DCOLOR_RGBA(r, g, b, a) D3DCOLOR_ARGB (a, r, g, b)
#define D3DCOLOR_XRGB(r, g, b) D3DCOLOR_ARGB (0xff, r, g, b)

/* DirectInput value types */
#include "dik.inc"
#include "sdkconst.inc"
#define DIRECTINPUT_VERSION 0x0700
#define DI_OK S_OK
#define DI_NOEFFECT S_FALSE
typedef struct DIJOYSTATE { LONG lX, lY, lZ, lRx, lRy, lRz, rglSlider[2]; DWORD rgdwPOV[4]; BYTE rgbButtons[32]; } DIJOYSTATE, *LPDIJOYSTATE;
typedef struct DIJOYSTATE2 { LONG lX, lY, lZ, lRx, lRy, lRz, rglSlider[2]; DWORD rgdwPOV[4]; BYTE rgbButtons[128];
	LONG lVX, lVY, lVZ, lVRx, lVRy, lVRz, rglVSlider[2], lAX, lAY, lAZ, lARx, lARy, lARz, rglASlider[2], lFX, lFY, lFZ, lFRx, lFRy, lFRz, rglFSlider[2];
} DIJOYSTATE2, *LPDIJOYSTATE2;
typedef struct DIDEVICEOBJECTDATA { DWORD dwOfs, dwData, dwTimeStamp, dwSequence; UINT_PTR uAppData; } DIDEVICEOBJECTDATA, *LPDIDEVICEOBJECTDATA;
typedef struct DIPROPHEADER { DWORD dwSize, dwHeaderSize, dwObj, dwHow; } DIPROPHEADER, *LPDIPROPHEADER;
typedef struct DIPROPDWORD { DIPROPHEADER diph; DWORD dwData; } DIPROPDWORD, *LPDIPROPDWORD;
typedef struct DIPROPRANGE { DIPROPHEADER diph; LONG lMin, lMax; } DIPROPRANGE, *LPDIPROPRANGE;
typedef struct DIDEVICEINSTANCE { DWORD dwSize; GUID guidInstance, guidProduct; DWORD dwDevType; CHAR tszInstanceName[MAX_PATH],
	tszProductName[MAX_PATH]; GUID guidFFDriver; WORD wUsagePage, wUsage; } DIDEVICEINSTANCE, *LPDIDEVICEINSTANCE;
typedef const DIDEVICEINSTANCE *LPCDIDEVICEINSTANCE;
typedef struct DIDEVICEOBJECTINSTANCE { DWORD dwSize; GUID guidType; DWORD dwOfs, dwType, dwFlags; CHAR tszName[MAX_PATH]; } DIDEVICEOBJECTINSTANCE,
	*LPDIDEVICEOBJECTINSTANCE;
typedef const DIDEVICEOBJECTINSTANCE *LPCDIDEVICEOBJECTINSTANCE;
typedef struct DIDEVCAPS { DWORD dwSize, dwFlags, dwDevType, dwAxes, dwButtons, dwPOVs, dwFFSamplePeriod, dwFFMinTimeResolution, dwFirmwareRevision,
	dwHardwareRevision, dwFFDriverVersion; } DIDEVCAPS, *LPDIDEVCAPS;
typedef struct DIEFFECT { DWORD dwSize, dwFlags, dwDuration, dwSamplePeriod, dwGain, dwTriggerButton, dwTriggerRepeatInterval, cAxes; LPDWORD rgdwAxes;
	LPLONG rglDirection; void *lpEnvelope; DWORD cbTypeSpecificParams; LPVOID lpvTypeSpecificParams; DWORD dwStartDelay; } DIEFFECT, *LPDIEFFECT;
typedef struct DICONSTANTFORCE { LONG lMagnitude; } DICONSTANTFORCE;
typedef struct DIPERIODIC { DWORD dwMagnitude; LONG lOffset; DWORD dwPhase, dwPeriod; } DIPERIODIC;
typedef struct DIDATAFORMAT { DWORD dwSize, dwObjSize, dwFlags, dwDataSize, dwNumObjs; void *rgodf; } DIDATAFORMAT, *LPDIDATAFORMAT;
extern const DIDATAFORMAT c_dfDIKeyboard, c_dfDIMouse, c_dfDIJoystick, c_dfDIJoystick2;
extern const GUID GUID_SysKeyboard, GUID_SysMouse, GUID_XAxis, GUID_YAxis, GUID_ZAxis, GUID_RxAxis, GUID_RyAxis, GUID_RzAxis, GUID_Slider,
	GUID_ConstantForce, GUID_Sine, GUID_Square, GUID_Triangle;
extern const GUID IID_IDirectInput7, IID_IDirectInputDevice7, IID_IDirectInputDevice2, IID_IDirectPlay4A, IID_IDirectPlayLobby3A,
	IID_IDirectDraw7, IID_IDirect3D9;
#define DIPROP_BUFFERSIZE ((const GUID *) 1)
#define DIPROP_RANGE ((const GUID *) 4)
#define DIPROP_DEADZONE ((const GUID *) 5)
#define DIPROP_SATURATION ((const GUID *) 6)
#define DIPROP_AUTOCENTER ((const GUID *) 9)
#define DIPH_DEVICE 0
#define DIPH_BYOFFSET 1
#define DIPH_BYID 2
#define DISCL_EXCLUSIVE 1
#define DISCL_NONEXCLUSIVE 2
#define DISCL_FOREGROUND 4
#define DISCL_BACKGROUND 8
#define DIEDFL_ALLDEVICES 0
#define DIEDFL_ATTACHEDONLY 1
#define DIEDFL_FORCEFEEDBACK 0x100
#define DIENUM_STOP 0
#define DIENUM_CONTINUE 1
#define DIDEVTYPE_JOYSTICK 4
#define DIDEVTYPE_HID 0x10000
#define DIDFT_ALL 0
#define DIDFT_AXIS 3
#define DIDFT_BUTTON 0x0c
#define DIDFT_POV 0x10
#define DIJOFS_X 0
#define DIJOFS_Y 4
#define DIJOFS_Z 8
#define DIJOFS_RX 12
#define DIJOFS_RY 16
#define DIJOFS_RZ 20
#define DIJOFS_SLIDER(n) (24 + (n) * 4)
#define DIJOFS_POV(n) (32 + (n) * 4)
#define DIJOFS_BUTTON(n) (48 + (n))
#define DIERR_INPUTLOST ((HRESULT) 0x8007001EL)
#define DIERR_NOTACQUIRED ((HRESULT) 0x8007000CL)
#define DIERR_OTHERAPPHASPRIO ((HRESULT) 0x80070005L)
#define DI_BUFFEROVERFLOW S_FALSE
#define DI8DEVTYPE_JOYSTICK 0x14
#define GET_DIDEVICE_TYPE(t) LOBYTE (t)

/* DirectSound value types */
typedef struct WAVEFORMATEX { WORD wFormatTag, nChannels; DWORD nSamplesPerSec, nAvgBytesPerSec; WORD nBlockAlign, wBitsPerSample, cbSize; } WAVEFORMATEX,
	*LPWAVEFORMATEX, *PWAVEFORMATEX;
typedef struct PCMWAVEFORMAT { struct { WORD wFormatTag, nChannels; DWORD nSamplesPerSec, nAvgBytesPerSec; WORD nBlockAlign; } wf; WORD wBitsPerSample; }
	PCMWAVEFORMAT;
typedef struct DSBUFFERDESC { DWORD dwSize, dwFlags, dwBufferBytes, dwReserved; LPWAVEFORMATEX lpwfxFormat; GUID guid3DAlgorithm; } DSBUFFERDESC,
	*LPDSBUFFERDESC;
typedef struct DSCAPS { DWORD dwSize, dwFlags, dwMinSecondarySampleRate, dwMaxSecondarySampleRate, dwPrimaryBuffers, dwMaxHwMixingAllBuffers,
	dwMaxHwMixingStaticBuffers, dwMaxHwMixingStreamingBuffers, dwFreeHwMixingAllBuffers, dwFreeHwMixingStaticBuffers, dwFreeHwMixingStreamingBuffers,
	dwMaxHw3DAllBuffers, dwMaxHw3DStaticBuffers, dwMaxHw3DStreamingBuffers, dwFreeHw3DAllBuffers, dwFreeHw3DStaticBuffers, dwFreeHw3DStreamingBuffers,
	dwTotalHwMemBytes, dwFreeHwMemBytes, dwMaxContigFreeHwMemBytes, dwUnlockTransferRateHwBuffers, dwPlayCpuOverheadSwBuffers, dwReserved1, dwReserved2;
} DSCAPS, *LPDSCAPS;
typedef struct DSBCAPS { DWORD dwSize, dwFlags, dwBufferBytes, dwUnlockTransferRate, dwPlayCpuOverhead; } DSBCAPS, *LPDSBCAPS;
#define WAVE_FORMAT_PCM 1
#define DS_OK S_OK

/* DirectPlay value types */
typedef DWORD DPID, *LPDPID;
typedef struct DPCAPS { DWORD dwSize, dwFlags, dwMaxBufferSize, dwMaxQueueSize, dwMaxPlayers, dwHundredBaud, dwLatency, dwMaxLocalPlayers,
	dwHeaderLength, dwTimeout; } DPCAPS, *LPDPCAPS;
typedef struct DPSESSIONDESC2 { DWORD dwSize, dwFlags; GUID guidInstance, guidApplication; DWORD dwMaxPlayers, dwCurrentPlayers;
	union { LPSTR lpszSessionNameA; LPWSTR lpszSessionName; }; union { LPSTR lpszPasswordA; LPWSTR lpszPassword; };
	DWORD_PTR dwReserved1, dwReserved2, dwUser1, dwUser2, dwUser3, dwUser4; } DPSESSIONDESC2, *LPDPSESSIONDESC2;
typedef const DPSESSIONDESC2 *LPCDPSESSIONDESC2;
typedef struct DPNAME { DWORD dwSize, dwFlags; union { LPSTR lpszShortNameA; LPWSTR lpszShortName; }; union { LPSTR lpszLongNameA; LPWSTR lpszLongName; };
} DPNAME, *LPDPNAME;
typedef const DPNAME *LPCDPNAME;
typedef struct DPMSG_GENERIC { DWORD dwType; } DPMSG_GENERIC, *LPDPMSG_GENERIC;
typedef struct DPMSG_CREATEPLAYERORGROUP { DWORD dwType, dwPlayerType; DPID dpId; DWORD dwCurrentPlayers; LPVOID lpData; DWORD dwDataSize; DPNAME dpnName;
	DPID dpIdParent; DWORD dwFlags; } DPMSG_CREATEPLAYERORGROUP, *LPDPMSG_CREATEPLAYERORGROUP;
typedef struct DPMSG_DESTROYPLAYERORGROUP { DWORD dwType, dwPlayerType; DPID dpId; LPVOID lpLocalData; DWORD dwLocalDataSize; LPVOID lpRemoteData;
	DWORD dwRemoteDataSize; DPNAME dpnName; DPID dpIdParent; DWORD dwFlags; } DPMSG_DESTROYPLAYERORGROUP, *LPDPMSG_DESTROYPLAYERORGROUP;
typedef struct DPLCONNECTION { DWORD dwSize, dwFlags; LPDPSESSIONDESC2 lpSessionDesc; LPDPNAME lpPlayerName; GUID guidSP; LPVOID lpAddress;
	DWORD dwAddressSize; } DPLCONNECTION, *LPDPLCONNECTION;
typedef struct DPCOMPOUNDADDRESSELEMENT { GUID guidDataType; DWORD dwDataSize; LPVOID lpData; } DPCOMPOUNDADDRESSELEMENT, *LPDPCOMPOUNDADDRESSELEMENT;
typedef struct DPLAPPINFO { DWORD dwSize; GUID guidApplication; union { LPSTR lpszAppNameA; LPWSTR lpszAppName; }; } DPLAPPINFO, *LPDPLAPPINFO;
typedef const DPLAPPINFO *LPCDPLAPPINFO;
typedef struct DPLMSG_GENERIC { DWORD dwType; } DPLMSG_GENERIC, *LPDPLMSG_GENERIC;
#define DP_OK S_OK
#define DPID_ALLPLAYERS 0
#define DPID_SYSMSG 0

extern const GUID DPSPGUID_IPX, DPSPGUID_MODEM, DPSPGUID_SERIAL, DPSPGUID_TCPIP, DPAID_ComPort, DPAID_INet, DPAID_Modem, DPAID_Phone,
	DPAID_ServiceProvider, CLSID_AMMultiMediaStream, CLSID_DirectPlay, CLSID_DirectPlayLobby, IID_IAMMultiMediaStream, IID_IDirect3D7,
	IID_IDirect3DHALDevice, IID_IDirectDrawMediaStream, IID_IDirectSound3DBuffer, IID_IDirectSound3DListener;
typedef struct _D3DADAPTER_IDENTIFIER9 { char Driver[512], Description[512], DeviceName[32]; LARGE_INTEGER DriverVersion; DWORD VendorId, DeviceId,
	SubSysId, Revision; GUID DeviceIdentifier; DWORD WHQLLevel; } D3DADAPTER_IDENTIFIER9;
typedef struct tagKERNINGPAIR { WORD wFirst, wSecond; int iKernAmount; } KERNINGPAIR, *LPKERNINGPAIR;

/* Win32 */
typedef struct _WIN32_FIND_DATAA { DWORD dwFileAttributes; FILETIME ftCreationTime, ftLastAccessTime, ftLastWriteTime; DWORD nFileSizeHigh, nFileSizeLow,
	dwReserved0, dwReserved1; CHAR cFileName[MAX_PATH]; CHAR cAlternateFileName[14]; } WIN32_FIND_DATA, WIN32_FIND_DATAA, *LPWIN32_FIND_DATA;
typedef struct tagPALETTEENTRY { BYTE peRed, peGreen, peBlue, peFlags; } PALETTEENTRY, *LPPALETTEENTRY;
typedef LRESULT (*WNDPROC) (HWND, UINT, WPARAM, LPARAM);
typedef struct tagWNDCLASSA { UINT style; WNDPROC lpfnWndProc; int cbClsExtra, cbWndExtra; HINSTANCE hInstance; HICON hIcon; HCURSOR hCursor;
	HBRUSH hbrBackground; LPCSTR lpszMenuName, lpszClassName; } WNDCLASS, WNDCLASSA, *LPWNDCLASS;
typedef struct timecaps_tag { UINT wPeriodMin, wPeriodMax; } TIMECAPS, *LPTIMECAPS;
typedef struct tagMCI_OPEN_PARMS { DWORD_PTR dwCallback; MCIDEVICEID wDeviceID; LPCSTR lpstrDeviceType, lpstrElementName, lpstrAlias; } MCI_OPEN_PARMS;
typedef struct tagMCI_PLAY_PARMS { DWORD_PTR dwCallback; DWORD dwFrom, dwTo; } MCI_PLAY_PARMS;
typedef struct tagMCI_STATUS_PARMS { DWORD_PTR dwCallback; DWORD_PTR dwReturn; DWORD dwItem, dwTrack; } MCI_STATUS_PARMS;
typedef struct tagMCI_SET_PARMS { DWORD_PTR dwCallback; DWORD dwTimeFormat, dwAudio; } MCI_SET_PARMS;
typedef struct tagMCI_GENERIC_PARMS { DWORD_PTR dwCallback; } MCI_GENERIC_PARMS;
typedef struct _SECURITY_ATTRIBUTES { DWORD nLength; LPVOID lpSecurityDescriptor; BOOL bInheritHandle; } SECURITY_ATTRIBUTES, *LPSECURITY_ATTRIBUTES;
typedef struct _OVERLAPPED { ULONG_PTR Internal, InternalHigh; DWORD Offset, OffsetHigh; HANDLE hEvent; } OVERLAPPED, *LPOVERLAPPED;
typedef struct _MEMORYSTATUS { DWORD dwLength, dwMemoryLoad; SIZE_T dwTotalPhys, dwAvailPhys, dwTotalPageFile, dwAvailPageFile, dwTotalVirtual,
	dwAvailVirtual; } MEMORYSTATUS, *LPMEMORYSTATUS;
typedef struct tagPAINTSTRUCT { HDC hdc; BOOL fErase; RECT rcPaint; BOOL fRestore, fIncUpdate; BYTE rgbReserved[32]; } PAINTSTRUCT;
typedef struct tagBITMAPINFOHEADER { DWORD biSize; LONG biWidth, biHeight; WORD biPlanes, biBitCount; DWORD biCompression, biSizeImage;
	LONG biXPelsPerMeter, biYPelsPerMeter; DWORD biClrUsed, biClrImportant; } BITMAPINFOHEADER, *LPBITMAPINFOHEADER;
typedef struct tagRGBQUAD { BYTE rgbBlue, rgbGreen, rgbRed, rgbReserved; } RGBQUAD;
typedef struct tagBITMAPINFO { BITMAPINFOHEADER bmiHeader; RGBQUAD bmiColors[1]; } BITMAPINFO, *LPBITMAPINFO;
typedef struct _OSVERSIONINFOA { DWORD dwOSVersionInfoSize, dwMajorVersion, dwMinorVersion, dwBuildNumber, dwPlatformId; CHAR szCSDVersion[128]; }
	OSVERSIONINFO, *LPOSVERSIONINFO;
typedef DWORD (WINAPI *LPTHREAD_START_ROUTINE) (LPVOID);
typedef void (CALLBACK *LPTIMECALLBACK) (UINT, UINT, DWORD_PTR, DWORD_PTR, DWORD_PTR);
typedef void (CALLBACK *TIMERPROC) (HWND, UINT, UINT_PTR, DWORD);

#define FILE_MAP_COPY 0x01
#define FILE_MAP_WRITE 0x02
#define FILE_MAP_READ 0x04
#define FILE_MAP_ALL_ACCESS 0xF001F
#define BI_RGB 0
#define HWND_TOP ((HWND) 0)
#define HWND_TOPMOST ((HWND) -1)
#define HWND_NOTOPMOST ((HWND) -2)
#define SW_SHOWDEFAULT 10
#define WS_VSCROLL 0x200000
#define WS_HSCROLL 0x100000
#define QS_ALLINPUT 0x04FF
#define OUT_TT_PRECIS 4
#define SB_VERT 1
#define SB_HORZ 0
typedef struct tagBITMAPFILEHEADER { WORD bfType; DWORD bfSize; WORD bfReserved1, bfReserved2; DWORD bfOffBits; } __attribute__ ((packed))
	BITMAPFILEHEADER;
#define INVALID_HANDLE_VALUE ((HANDLE) (LONG_PTR) -1)
#define WM_NULL 0x0000
#define WM_CREATE 0x0001
#define WM_DESTROY 0x0002
#define WM_MOVE 0x0003
#define WM_SIZE 0x0005
#define WM_ACTIVATE 0x0006
#define WM_SETFOCUS 0x0007
#define WM_KILLFOCUS 0x0008
#define WM_PAINT 0x000F
#define WM_CLOSE 0x0010
#define WM_QUIT 0x0012
#define WM_ERASEBKGND 0x0014
#define WM_SHOWWINDOW 0x0018
#define WM_ACTIVATEAPP 0x001C
#define WM_SETCURSOR 0x0020
#define WM_GETMINMAXINFO 0x0024
#define WM_KEYDOWN 0x0100
#define WM_KEYUP 0x0101
#define WM_CHAR 0x0102
#define WM_SYSKEYDOWN 0x0104
#define WM_SYSKEYUP 0x0105
#define WM_SYSCHAR 0x0106
#define WM_COMMAND 0x0111
#define WM_SYSCOMMAND 0x0112
#define WM_TIMER 0x0113
#define WM_MOUSEMOVE 0x0200
#define WM_LBUTTONDOWN 0x0201
#define WM_LBUTTONUP 0x0202
#define WM_RBUTTONDOWN 0x0204
#define WM_RBUTTONUP 0x0205
#define WM_MBUTTONDOWN 0x0207
#define WM_MBUTTONUP 0x0208
#define WM_MOUSEWHEEL 0x020A
#define WM_USER 0x0400
#define WM_APP 0x8000
#define TIME_ONESHOT 0
#define TIME_PERIODIC 1
#define TIMERR_NOERROR 0
#define TIMERR_NOCANDO 97
#define THREAD_PRIORITY_IDLE (-15)
#define THREAD_PRIORITY_LOWEST (-2)
#define THREAD_PRIORITY_BELOW_NORMAL (-1)
#define THREAD_PRIORITY_NORMAL 0
#define THREAD_PRIORITY_ABOVE_NORMAL 1
#define THREAD_PRIORITY_HIGHEST 2
#define THREAD_PRIORITY_TIME_CRITICAL 15
#define PAGE_NOACCESS 0x01
#define PAGE_READONLY 0x02
#define PAGE_READWRITE 0x04
#define MEM_COMMIT 0x1000
#define MEM_RESERVE 0x2000
#define MEM_DECOMMIT 0x4000
#define MEM_RELEASE 0x8000
#define MEM_TOP_DOWN 0x100000
#define GENERIC_READ 0x80000000u
#define GENERIC_WRITE 0x40000000u
#define FILE_SHARE_READ 1
#define FILE_SHARE_WRITE 2
#define CREATE_NEW 1
#define CREATE_ALWAYS 2
#define OPEN_EXISTING 3
#define OPEN_ALWAYS 4
#define FILE_ATTRIBUTE_READONLY 0x01
#define FILE_ATTRIBUTE_HIDDEN 0x02
#define FILE_ATTRIBUTE_DIRECTORY 0x10
#define FILE_ATTRIBUTE_NORMAL 0x80
#define FILE_BEGIN 0
#define FILE_CURRENT 1
#define FILE_END 2
#define MCI_OPEN 0x0803
#define MCI_CLOSE 0x0804
#define MCI_PLAY 0x0806
#define MCI_STOP 0x0808
#define MCI_PAUSE 0x0809
#define MCI_SET 0x080D
#define MCI_STATUS 0x0814
#define MCI_NOTIFY 0x1
#define MCI_WAIT 0x2
#define MCI_FROM 0x4
#define MCI_TO 0x8
#define MCI_TRACK 0x10
#define MCI_STATUS_ITEM 0x100
#define MCI_STATUS_LENGTH 1
#define MCI_STATUS_POSITION 2
#define MCI_STATUS_NUMBER_OF_TRACKS 3
#define MCI_OPEN_TYPE 0x2000
#define MCI_OPEN_ELEMENT 0x200
#define MCI_SET_TIME_FORMAT 0x400
#define MCI_FORMAT_TMSF 10
#define MCI_FORMAT_MILLISECONDS 0
#define MCI_MAKE_TMSF(t, m, s, f) ((DWORD) (((BYTE) (t) | ((WORD) (m) << 8)) | ((DWORD) (BYTE) (s) | ((WORD) (f) << 8)) << 16))
#define MB_OK 0
#define MB_OKCANCEL 1
#define MB_YESNO 4
#define MB_ICONERROR 0x10
#define MB_ICONSTOP 0x10
#define MB_ICONQUESTION 0x20
#define MB_ICONWARNING 0x30
#define MB_ICONINFORMATION 0x40
#define MB_SYSTEMMODAL 0x1000
#define MB_TOPMOST 0x40000
#define MB_SETFOREGROUND 0x10000
#define IDOK 1
#define IDCANCEL 2
#define IDYES 6
#define IDNO 7
#define SW_HIDE 0
#define SW_SHOWNORMAL 1
#define SW_SHOW 5
#define SW_MINIMIZE 6
#define SW_RESTORE 9
#define WAIT_OBJECT_0 0
#define WAIT_TIMEOUT 258
#define WAIT_FAILED 0xFFFFFFFFu
#define ERROR_SUCCESS 0
#define ERROR_ALREADY_EXISTS 183
#define STD_OUTPUT_HANDLE ((DWORD) -11)
#define STD_ERROR_HANDLE ((DWORD) -12)
#define CS_HREDRAW 2
#define CS_VREDRAW 1
#define CS_OWNDC 0x20
#define WS_OVERLAPPED 0
#define WS_POPUP 0x80000000u
#define WS_VISIBLE 0x10000000u
#define WS_CAPTION 0xC00000
#define WS_SYSMENU 0x80000
#define WS_THICKFRAME 0x40000
#define WS_MINIMIZEBOX 0x20000
#define WS_MAXIMIZEBOX 0x10000
#define WS_OVERLAPPEDWINDOW (WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX)
#define WS_EX_TOPMOST 8
#define WS_EX_APPWINDOW 0x40000
#define CW_USEDEFAULT ((int) 0x80000000)
#define GWL_STYLE (-16)
#define GWL_EXSTYLE (-20)
#define IDC_ARROW ((LPCSTR) 32512)
#define IDI_APPLICATION ((LPCSTR) 32512)
#define BLACK_BRUSH 4
#define PM_NOREMOVE 0
#define PM_REMOVE 1
#define VK_ESCAPE 0x1B
#define VK_RETURN 0x0D
#define VK_SHIFT 0x10
#define VK_CONTROL 0x11
#define VK_MENU 0x12
#define VK_CAPITAL 0x14
#define VK_NUMLOCK 0x90
#define VK_SCROLL 0x91
#define HKEY_CLASSES_ROOT ((HKEY) (ULONG_PTR) 0x80000000u)
#define HKEY_CURRENT_USER ((HKEY) (ULONG_PTR) 0x80000001u)
#define HKEY_LOCAL_MACHINE ((HKEY) (ULONG_PTR) 0x80000002u)
#define KEY_READ 0x20019
#define KEY_WRITE 0x20006
#define KEY_ALL_ACCESS 0xF003F
#define REG_SZ 1
#define REG_DWORD 4
#define REG_BINARY 3
#define REG_OPTION_NON_VOLATILE 0
#define E_OUTOFMEMORY ((HRESULT) 0x8007000EL)
#define E_INVALIDARG ((HRESULT) 0x80070057L)
#define E_NOINTERFACE ((HRESULT) 0x80004002L)
#define E_POINTER ((HRESULT) 0x80004003L)
#define E_NOTIMPL ((HRESULT) 0x80004001L)
#define CLSCTX_INPROC_SERVER 1
#define CLSCTX_ALL 0x17
#define MAKEINTRESOURCE(i) ((LPSTR) (ULONG_PTR) ((WORD) (i)))
#define GetRValue(rgb) ((BYTE) (rgb))
#define GetGValue(rgb) ((BYTE) (((WORD) (rgb)) >> 8))
#define GetBValue(rgb) ((BYTE) ((rgb) >> 16))
#define max(a, b) (((a) > (b)) ? (a) : (b))
#define min(a, b) (((a) < (b)) ? (a) : (b))
#define _MAX_PATH MAX_PATH
#define _MAX_DRIVE 3
#define _MAX_DIR 256
#define _MAX_FNAME 256
#define _MAX_EXT 256

#endif
