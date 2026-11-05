/*
 * EECH headless build: the DirectX COM entry points (the C macros of the
 * DirectX headers, IDirectSound_CreateSoundBuffer and so on). EECH calls them
 * without prototypes, so they are defined without prototypes here: every one
 * fails with E_FAIL, which EECH's sound, input and 3D backends already treat
 * as "no device".
 */

#include <stdint.h>

/*
 * 64-bit, as long is on Linux (LP64), where the platform layer was developed:
 * there E_FAIL (0x80004005) is positive, so FAILED () is false for the null
 * COM stubs' E_FAIL, and EECH's Direct3D set-up takes its success path, which
 * a headless run needs. A 32-bit HRESULT (Windows' long) would make it fatal.
 */
typedef int64_t HRESULT;
#define E_FAIL ((HRESULT) 0x80004005L)

#define NULL_COM(name) HRESULT name () { return E_FAIL; }

#ifdef EECH_ENV_OBSERVER
/*
 * TEST-ONLY (feature env-observer, EEDCS-51): the four DirectSound entry points the application sound system calls while it initialises count their calls and the low bits of what they return
 * (eech_env_observer.c, the sound initialisation probe), so the report can say which one failed without a call of its own. They return exactly what NULL_COM returns.
 */
extern void eech_envobs_com (int which, long long returned);

#define COUNTED_COM(name, which) HRESULT name () { eech_envobs_com (which, (long long) E_FAIL); return E_FAIL; }
#endif

NULL_COM (DirectInputCreateEx)
#ifdef EECH_ENV_OBSERVER
COUNTED_COM (DirectSoundCreate, 0)
#else
NULL_COM (DirectSoundCreate)
#endif
NULL_COM (IAMMultiMediaStream_AddMediaStream)
NULL_COM (IAMMultiMediaStream_GetMediaStream)
NULL_COM (IAMMultiMediaStream_Initialize)
NULL_COM (IAMMultiMediaStream_OpenFile)
NULL_COM (IAMMultiMediaStream_Release)
NULL_COM (IAMMultiMediaStream_SetState)
NULL_COM (IDirect3D7_CreateDevice)
NULL_COM (IDirect3D7_Release)
NULL_COM (IDirect3D9_CreateDevice)
NULL_COM (IDirect3DDevice7_GetCaps)
NULL_COM (IDirect3DDevice7_Release)
NULL_COM (IDirect3DDevice7_SetRenderTarget)
NULL_COM (IDirect3DDevice9_DrawPrimitiveUP)
NULL_COM (IDirect3DDevice9_GetDeviceCaps)
NULL_COM (IDirect3DDevice9_GetRenderState)
NULL_COM (IDirect3DDevice9_GetTextureStageState)
NULL_COM (IDirect3DDevice9_Release)
NULL_COM (IDirect3DDevice9_Reset)
NULL_COM (IDirect3DDevice9_SetFVF)
NULL_COM (IDirect3DDevice9_TestCooperativeLevel)
NULL_COM (IDirect3DTexture9_Release)
NULL_COM (IDirectDraw7_QueryInterface)
NULL_COM (IDirectDrawMediaStream_CreateSample)
NULL_COM (IDirectDrawStreamSample_GetSurface)
NULL_COM (IDirectDrawStreamSample_Release)
NULL_COM (IDirectDrawStreamSample_SetRect)
NULL_COM (IDirectDrawStreamSample_Update)
NULL_COM (IDirectDrawSurface7_Blt)
NULL_COM (IDirectInput7_CreateDeviceEx)
NULL_COM (IDirectInput7_Release)
NULL_COM (IDirectInputDevice7_Acquire)
NULL_COM (IDirectInputDevice7_GetDeviceData)
NULL_COM (IDirectInputDevice7_GetDeviceState)
NULL_COM (IDirectInputDevice7_Release)
NULL_COM (IDirectInputDevice7_SetCooperativeLevel)
NULL_COM (IDirectInputDevice7_SetDataFormat)
NULL_COM (IDirectInputDevice7_SetEventNotification)
NULL_COM (IDirectInputDevice7_SetProperty)
NULL_COM (IDirectInputDevice7_Unacquire)
NULL_COM (IDirectInputDevice_Acquire)
NULL_COM (IDirectInputDevice_GetDeviceData)
NULL_COM (IDirectSound3DBuffer_GetMaxDistance)
NULL_COM (IDirectSound3DBuffer_GetMinDistance)
NULL_COM (IDirectSound3DBuffer_GetPosition)
NULL_COM (IDirectSound3DBuffer_GetVelocity)
NULL_COM (IDirectSound3DBuffer_SetMaxDistance)
NULL_COM (IDirectSound3DBuffer_SetMinDistance)
NULL_COM (IDirectSound3DBuffer_SetPosition)
NULL_COM (IDirectSound3DBuffer_SetVelocity)
NULL_COM (IDirectSound3DListener_CommitDeferredSettings)
NULL_COM (IDirectSound3DListener_GetDopplerFactor)
NULL_COM (IDirectSound3DListener_GetOrientation)
NULL_COM (IDirectSound3DListener_GetPosition)
NULL_COM (IDirectSound3DListener_GetRolloffFactor)
NULL_COM (IDirectSound3DListener_GetVelocity)
NULL_COM (IDirectSound3DListener_SetDopplerFactor)
NULL_COM (IDirectSound3DListener_SetOrientation)
NULL_COM (IDirectSound3DListener_SetPosition)
NULL_COM (IDirectSound3DListener_SetRolloffFactor)
NULL_COM (IDirectSound3DListener_SetVelocity)
NULL_COM (IDirectSoundBuffer_GetCaps)
NULL_COM (IDirectSoundBuffer_GetCurrentPosition)
NULL_COM (IDirectSoundBuffer_GetFrequency)
NULL_COM (IDirectSoundBuffer_GetPan)
NULL_COM (IDirectSoundBuffer_GetStatus)
NULL_COM (IDirectSoundBuffer_GetVolume)
NULL_COM (IDirectSoundBuffer_Lock)
NULL_COM (IDirectSoundBuffer_Play)
NULL_COM (IDirectSoundBuffer_QueryInterface)
NULL_COM (IDirectSoundBuffer_Release)
NULL_COM (IDirectSoundBuffer_SetCurrentPosition)
NULL_COM (IDirectSoundBuffer_SetFrequency)
NULL_COM (IDirectSoundBuffer_SetPan)
NULL_COM (IDirectSoundBuffer_SetVolume)
NULL_COM (IDirectSoundBuffer_Stop)
NULL_COM (IDirectSoundBuffer_Unlock)
#ifdef EECH_ENV_OBSERVER
COUNTED_COM (IDirectSound_CreateSoundBuffer, 2)
#else
NULL_COM (IDirectSound_CreateSoundBuffer)
#endif
NULL_COM (IDirectSound_DuplicateSoundBuffer)
#ifdef EECH_ENV_OBSERVER
COUNTED_COM (IDirectSound_Release, 3)
#else
NULL_COM (IDirectSound_Release)
#endif
#ifdef EECH_ENV_OBSERVER
COUNTED_COM (IDirectSound_SetCooperativeLevel, 1)
#else
NULL_COM (IDirectSound_SetCooperativeLevel)
#endif
NULL_COM (IMediaStream_QueryInterface)
NULL_COM (IMediaStream_Release)
