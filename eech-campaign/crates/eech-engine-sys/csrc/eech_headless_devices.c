/*
 * EECH headless build: null devices. Replaces the backends that need a real
 * Windows machine and have nothing to do headless:
 *
 *   modules/system/joystick.c        DirectInput joysticks and force feedback: none attached
 *   modules/userint2/ui_draw/uifont.c GDI glyph rasterisation: no fonts drawn
 *   modules/graphics/dirdraw.c       display devices and modes: none
 *   modules/graphics/f3d.c           the Direct3D 9 device: none (every call is a no-op)
 *   modules/multi/directp.c          DirectPlay: no connection (single player, local server)
 *   modules/multi/dpguid.c           DirectPlay GUIDs
 *
 * and the DirectX COM entry points the other backends call, which fail with
 * E_FAIL so those backends report "no device" through their own error paths.
 */

#include "system.h"
#include "graphics.h"
#include "multi.h"
#include "userint2.h"

#include "eech_headless.h"

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* joystick.c: no devices */

int number_of_joystick_devices;
joystick_device_info *joystick_devices;
float xforce, yforce, xfreq, yfreq, xampl, yampl, ff_xtrim, ff_ytrim, ff_xcorr, ff_ycorr;
LPDIRECTINPUTDEVICE7 feedback_device;
AxisInfo_t AxisInfo[16 * 8];
int AxisCount;

void initialise_joysticks (void)
{
	number_of_joystick_devices = 0;
}

void deinitialise_joysticks (void)
{
}

void read_joystick_values (int joystick_device_index)
{
	(void) joystick_device_index;
}

int get_joystick_axis (int joystick_index, int axis)
{
	(void) joystick_index;
	(void) axis;
	return 0;
}

joystick_hat_position get_joystick_hat (joystick_device_info *stick, int index)
{
	(void) stick;
	(void) index;
	return (joystick_hat_position) 0;
}

void change_joystick_properties (void)
{
}

void set_joystick_force_feedback_forces ()
{
}

void play_ffb_weapon_effect (char *eff_name, float rate)
{
	(void) eff_name;
	(void) rate;
}

void GetGUIDString (const int device, char *text)
{
	(void) device;
	text[0] = 0;
}

void ShutdownAxisInformation (void)
{
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* uifont.c: no glyphs; string widths are zero and heights a nominal 10 pixels */

ui_font_type ui_fonts[NUM_FONT_TYPES];
ui_font_type *current_font = &ui_fonts[0];
rgb_colour ui_default_colour, ui_white_colour, ui_black_colour, ui_grey_colour, ui_blue_colour, ui_green_colour, ui_orange_colour, ui_red_colour,
	ui_yellow_colour;

static font_types current_font_type, stored_font_type;

void set_ui_font_resolution (int width, int height)
{
	(void) width;
	(void) height;
}

void initialise_ui_font (void)
{
	for (int i = 0; i < NUM_FONT_TYPES; i++)
	{
		ui_fonts[i].font_height = 10;
	}
	current_font = &ui_fonts[0];
}

void reinitialise_ui_fonts (void)
{
}

void deinitialise_ui_font (void)
{
}

float ui_display_text (const char *text, float x, float y)
{
	(void) text;
	(void) y;
	return x;
}

float ui_get_string_length (const char *string)
{
	(void) string;
	return 0.0f;
}

void set_ui_font_type (font_types font)
{
	current_font_type = font;
	current_font = &ui_fonts[font < NUM_FONT_TYPES ? font : 0];
}

void set_ui_font_colour (rgb_colour colour)
{
	(void) colour;
}

font_types get_ui_font_type (void)
{
	return current_font_type;
}

void ui_save_current_font (void)
{
	stored_font_type = current_font_type;
}

void ui_restore_current_font (void)
{
	set_ui_font_type (stored_font_type);
}

void ui_set_object_font (ui_object *obj)
{
	(void) obj;
}

/*
 * string_to_utf8 is text conversion, not drawing: ported from uifont.c
 * (the non-Polish character tables) because the language database uses it.
 */

static const unsigned char font_character_table[] =
{
	'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
	'1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '.', ',', ':', ';', '\\','/', '_', '-', '+', '*', '%', '!', '(', ')', '\'', '?',
	0xc4, 0xd6, 0xdc, 0xdf, 0xe4, 0xf6, 0xfc, '>', '<', ' ', 0xbf, 0xc0, 0xc1, 0xc2, 0xc3, 0xc7, 0xc8, 0xc9, 0xca, 0xcc, 0xcd, 0xce, 0xd1, 0xd2, 0xd3, 0xd4,
	0xd5, 0xd9, 0xda, 0xdb, 0xdd, 0xe0, 0xe1, 0xe2, 0xe3, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf9, 0xfa, 0xfb, 0xfd,
	'#', '[', ']', '=', 0x9c, '\"', 0xa3, '$', '^', '&', '{', '}', 222,
	255,
	126, 127, 128, 129, 130, 131, 132, 133,
	134, 135, 136, 137, 138, 139, 140, 141,
	142, 143, 144, 145, 146, 147, 148, 149,
	150, 151, 152, 153, 154, 155, 156, 157,
	158, 159, 160, 161, 162, 163, 164, 165,
	166, 167, 168, 169, 170, 171, 172, 173,
	174, 175, 176, 177, 178, 179, 180, 181,
	182, 183, 184, 185, 186, 187, 188, 189,
	190,
};

static const unsigned int unicode_character_table[] =
{
	0x0425, 0x0427, 0x044a, 0x0426, 0x0428, 0x0424, 0x042d, 0x0419, 0x042b, 0x042f, 0x0416, 0x044d, 0x0449, 0x0448, 0x0445, 0x0423,
	0x0420, 0x0415, 0x042c, 0x0422, 0x0410, 0x041b, 0x041a, 0x041c, 0x041f, 0x0411, 0x044e, 0x0446, 0x0418, 0x043c, 0x0421, 0x0431,
	0x041e, 0x0439, 0x0436, 0x0414, 0x044b, 0x0413, 0x0444, 0x0432, 0x0442, 0x0434, 0x043e, 0x043f, 0x043d, 0x044c, 0x043b, 0x0447,
	0x041d, 0x044f, 0x0438, 0x0441, 0x0435, 0x0412, 0x043a, 0x0437, 0x0443, 0x0440, 0x0433, 0x0430, 0x0417, 0x2026, 0xfeff, 0x042e,
	0x0451
};

static char unicode_table[256][2];
static int unicode_table_filled;

char *string_to_utf8 (const char *str)
{
	if (!unicode_table_filled)
	{
		for (unsigned count = 0; count < 256; count++)
		{
			unicode_table[count][0] = '~';
			unicode_table[count][1] = '\0';
		}
		for (unsigned count = 0; count < ARRAYSIZE (font_character_table); count++)
		{
			unsigned ch = font_character_table[count];
			unsigned rp = ch >= 126 && ch <= 190 ? unicode_character_table[ch - 126] : ch == 255 ? 0xA9 : ch;
			if (rp >= FONT_CHARACTERS)
			{
				rp = ' ';
			}
			if (rp < 0x80)
			{
				unicode_table[ch][0] = (char) rp;
				unicode_table[ch][1] = 0;
			}
			else
			{
				unicode_table[ch][0] = (char) (0xC0 | (rp >> 6));
				unicode_table[ch][1] = (char) (0x80 | (rp & 0x3F));
			}
		}
		unicode_table_filled = 1;
	}
	size_t len = 0;
	for (const char *src = str; *src; src++)
	{
		len += unicode_table[(unsigned char) *src][1] ? 2 : 1;
	}
	char *dst = (char *) safe_malloc (len + 1), *ptr = dst;
	for (; *str; str++)
	{
		const char *src = unicode_table[(unsigned char) *str];
		*ptr++ = src[0];
		if (src[1])
		{
			*ptr++ = src[1];
		}
	}
	*ptr = 0;
	return dst;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* dirdraw.c: no display */

int zbuffer_on_video, number_of_display_devices, number_display_modes;
display_device *display_devices;
display_format display_modes[1];
HWND export_window;

void set_ddraw_use_full_screen (int flag)
{
	(void) flag;
}

static int graphics_initialised;

int get_graphics_system_initialised (void)
{
	return graphics_initialised;
}

void get_ddraw_device_guid (GUID *guid)
{
	memset (guid, 0, sizeof (*guid));
}

/* dirdraw.c :: initialise_graphics_system, without the DirectDraw device */
BOOL initialise_graphics_system (GUID *device_guid)
{
	(void) device_guid;
	initialise_graphics_colours ();
	initialise_system_graphics_screens ();
	initialise_psd_layers ();
	graphics_initialised = TRUE;
	return TRUE;
}

/* dirdraw.c :: ddraw_set_display_resolution: the video screen is a memory surface */
BOOL ddraw_set_display_resolution (int width, int height)
{
	application_video_height = height;
	application_video_width = width;
	set_viewport (0, 0, width, height);
	create_video_screen (width, height);
	return TRUE;
}

BOOL ddraw_change_display_resolution (int width, int height)
{
	(void) width;
	(void) height;
	return FALSE;
}

BOOL ddraw_flip_surface (void)
{
	return TRUE;
}

void copy_export_mfd (screen *export_left, screen *export_right)
{
	(void) export_left;
	(void) export_right;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* f3d.c: no Direct3D device */

void f3d_set_light (unsigned index, struct _D3DLIGHT9 *light) { (void) index; (void) light; }
void f3d_light_enable (unsigned index, unsigned state) { (void) index; (void) state; }
void f3d_set_transform (unsigned type, struct _D3DMATRIX *matrix) { (void) type; (void) matrix; }
int f3d_set_viewport (struct _D3DVIEWPORT9 *viewport) { (void) viewport; return FALSE; }
void f3d_vertex_create (unsigned size, unsigned fvf, LPDIRECT3DVERTEXBUFFER9 *buffer) { (void) size; (void) fvf; *buffer = NULL; }
void f3d_vertex_release (LPDIRECT3DVERTEXBUFFER9 *buffer) { *buffer = NULL; }
void f3d_vertex_lock (LPDIRECT3DVERTEXBUFFER9 buffer, unsigned flags, void **ptr) { (void) buffer; (void) flags; *ptr = NULL; }
void f3d_vertex_unlock (LPDIRECT3DVERTEXBUFFER9 buffer) { (void) buffer; }
void f3d_index_create (unsigned size, LPDIRECT3DINDEXBUFFER9 *ibuffer) { (void) size; *ibuffer = NULL; }
void f3d_index_release (LPDIRECT3DINDEXBUFFER9 *ibuffer) { *ibuffer = NULL; }
void f3d_index_lock (LPDIRECT3DINDEXBUFFER9 ibuffer, unsigned flags, unsigned short **ptr) { (void) ibuffer; (void) flags; *ptr = NULL; }
void f3d_index_unlock (LPDIRECT3DINDEXBUFFER9 ibuffer) { (void) ibuffer; }
void f3d_dip (unsigned type, LPDIRECT3DVERTEXBUFFER9 buffer, unsigned vstart, unsigned vtotal, LPDIRECT3DINDEXBUFFER9 ibuffer, unsigned istart,
	unsigned primitives, unsigned fvf, unsigned stride)
{
	(void) type; (void) buffer; (void) vstart; (void) vtotal; (void) ibuffer; (void) istart; (void) primitives; (void) fvf; (void) stride;
}
void f3d_dp (unsigned type, LPDIRECT3DVERTEXBUFFER9 buffer, unsigned vstart, unsigned primitivecount, unsigned fvf, unsigned stride)
{
	(void) type; (void) buffer; (void) vstart; (void) primitivecount; (void) fvf; (void) stride;
}
void f3d_render_state (unsigned state, unsigned data) { (void) state; (void) data; }
void f3d_set_texture_state (unsigned stage, unsigned state, unsigned data) { (void) stage; (void) state; (void) data; }
void f3d_set_sampler_state (unsigned sampler, unsigned type, unsigned data) { (void) sampler; (void) type; (void) data; }
void f3d_set_texture (unsigned stage, struct SCREEN *texture) { (void) stage; (void) texture; }
void f3d_set_material (struct _D3DMATERIAL9 *material) { (void) material; }
int f3d_scene_begin (void) { return FALSE; }
int f3d_scene_end (void) { return FALSE; }
void f3d_clear_zbuffer (void) { }
void f3d_clear_screen (unsigned color) { (void) color; }
int f3d_set_3d_render_target (struct SCREEN *this_screen) { (void) this_screen; return FALSE; }
void f3d_stop_3d_render_target (struct SCREEN *this_screen) { (void) this_screen; }
/*
 * Surfaces are system memory: a texture or the video screen can be locked and
 * written (EECH's UI and texture loaders do) and nothing is displayed. The
 * opaque texture pointer holds the pixel levels.
 */

struct memory_texture
{
	unsigned int *levels[16];
	unsigned int width, height;
};

static unsigned int *memory_level (struct SCREEN *texture, int level)
{
	struct memory_texture *m = (struct memory_texture *) texture->texture;
	if (!m)
	{
		m = calloc (1, sizeof (*m));
		texture->texture = (LPDIRECT3DTEXTURE9) m;
	}
	if (level < 0 || level >= 16)
	{
		return NULL;
	}
	unsigned int w = texture->width >> level, h = texture->height >> level;
	w = w ? w : 1;
	h = h ? h : 1;
	if (!m->levels[level])
	{
		m->levels[level] = calloc ((size_t) w * h, sizeof (unsigned int));
	}
	return m->levels[level];
}

void f3d_texture_create (struct SCREEN *texture, int width, int height, int number_of_mipmaps, enum TEXTURE_ROLE role)
{
	(void) number_of_mipmaps;
	(void) role;
	texture->width = (unsigned int) width;
	texture->height = (unsigned int) height;
	texture->texture = NULL;
	memory_level (texture, 0);
}

void f3d_texture_release (struct SCREEN *texture)
{
	struct memory_texture *m = (struct memory_texture *) texture->texture;
	if (m)
	{
		for (int i = 0; i < 16; i++)
		{
			free (m->levels[i]);
		}
		free (m);
	}
	texture->texture = NULL;
	texture->data = NULL;
}

void f3d_texture_pre (struct SCREEN *texture) { (void) texture; }
void f3d_texture_post (struct SCREEN *texture) { (void) texture; }

int f3d_texture_lock (struct SCREEN *texture, int mipmap_level)
{
	unsigned int *pixels = memory_level (texture, mipmap_level);
	if (!pixels)
	{
		return FALSE;
	}
	unsigned int w = texture->width >> mipmap_level;
	texture->mipmap = (unsigned int) mipmap_level;
	texture->data = pixels;
	texture->pitch = w ? w : 1;
	return TRUE;
}

int f3d_texture_unlock (struct SCREEN *texture)
{
	texture->data = NULL;
	if (texture == video_screen)
	{
		/* the video screen keeps no texture between locks in EECH (f3d.c) */
		struct memory_texture *m = (struct memory_texture *) texture->texture;
		(void) m;
	}
	return TRUE;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* directp.c: no DirectPlay. The connection is never initialised, so EECH's comms run as a local server. */

direct_play_comms_mode_types direct_play_comms_mode = DIRECT_PLAY_COMMS_MODE_NONE;
modem_names *modem_name_list;
int direct_play_use_guaranteed_packets;

static connection_data_type connection;
static DPSESSIONDESC2 session_description;

connection_data_type *direct_play_get_connection_data (void) { return &connection; }
int direct_play_initialise_system (void) { return FALSE; }
void direct_play_register_application (const char *app_name, const char *filename) { (void) app_name; (void) filename; }
int direct_play_close_session (void) { return TRUE; }
int direct_play_create_interface (service_provider_table_type *this_service) { (void) this_service; return FALSE; }
int direct_play_destroy_interface (void) { return TRUE; }
int direct_play_interface_capabilities (void) { return FALSE; }
int direct_play_enumerate_sessions (void) { return FALSE; }
int direct_play_create_session (int value) { (void) value; return FALSE; }
int direct_play_join_session (void) { return FALSE; }
int direct_play_session_capabilities (void) { return FALSE; }
LPDPSESSIONDESC2 direct_play_get_session_capabilities (void) { return &session_description; }
int direct_play_session_max_players (void) { return 1; }
void direct_play_set_session_name (const char *name) { (void) name; }
void direct_play_set_session_type_and_name (int value) { (void) value; }
int direct_play_create_player (void) { return FALSE; }
int direct_play_destroy_player (void) { return TRUE; }
void direct_play_set_player_name (const char *name) { (void) name; }
int direct_play_get_player_id (void) { return 0; }
int direct_play_get_number_of_players (void) { return 1; }
int direct_play_create_group (void) { return FALSE; }
int direct_play_destroy_group (void) { return TRUE; }
int direct_play_join_group (void) { return FALSE; }
int direct_play_leave_group (void) { return TRUE; }
int direct_play_get_group_id (void) { return 0; }
int direct_play_enumerate_groups (void) { return FALSE; }
struct GROUP_TABLE_TYPE *direct_play_get_group_table (void) { return NULL; }
struct SESSION_TABLE_TYPE *direct_play_get_session_table (void) { return NULL; }
struct SERVICE_PROVIDER_TABLE_TYPE *direct_play_get_service_provider_table (void) { return NULL; }
int direct_play_send_data (DPID to_id, void *data, int size) { (void) to_id; (void) data; (void) size; return FALSE; }
int direct_play_receive_data (void *data, int size) { (void) data; (void) size; return 0; }
void direct_play_get_message_queue (void) { }
int direct_play_get_send_queue_number_of_packets (void) { return 0; }
int direct_play_get_connection_baudrate (void) { return 0; }
int direct_play_answer_modem (const char *modem, int user_data) { (void) modem; (void) user_data; return FALSE; }
int direct_play_dial_modem (const char *modem, const char *phone_number) { (void) modem; (void) phone_number; return FALSE; }
HRESULT direct_play_dial_modem_status (void) { return E_FAIL; }
void direct_play_destroy_modem (void) { }
int direct_play_refresh_modem_session (void) { return FALSE; }
void set_direct_play_inet_address (const char *address) { (void) address; }
void set_direct_play_serial_address (int com_port, int baud_rate, int stop_bits, int parity, int flow)
{
	(void) com_port; (void) baud_rate; (void) stop_bits; (void) parity; (void) flow;
}

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* GUIDs (dpguid.c, dinput, dsound): identities only, never compared against a real device */

const GUID DPSPGUID_IPX, DPSPGUID_MODEM, DPSPGUID_SERIAL, DPSPGUID_TCPIP, DPAID_ComPort, DPAID_INet, DPAID_Modem, DPAID_Phone,
	DPAID_ServiceProvider, CLSID_AMMultiMediaStream, CLSID_DirectPlay, CLSID_DirectPlayLobby, IID_IAMMultiMediaStream, IID_IDirect3D7,
	IID_IDirect3DHALDevice, IID_IDirectDrawMediaStream, IID_IDirectSound3DBuffer, IID_IDirectSound3DListener;
const GUID GUID_SysKeyboard, GUID_SysMouse, GUID_XAxis, GUID_YAxis, GUID_ZAxis, GUID_RxAxis, GUID_RyAxis, GUID_RzAxis, GUID_Slider,
	GUID_ConstantForce, GUID_Sine, GUID_Square, GUID_Triangle;
const GUID IID_IDirectInput7, IID_IDirectInputDevice7, IID_IDirectInputDevice2, IID_IDirectPlay4A, IID_IDirectPlayLobby3A, IID_IDirectDraw7,
	IID_IDirect3D9;
const DIDATAFORMAT c_dfDIKeyboard, c_dfDIMouse, c_dfDIJoystick, c_dfDIJoystick2;

/* ---------------------------------------------------------------------------------------------------------------------------- */
/* nedmalloc: the system allocator */

void *nedmalloc (size_t size) { return malloc (size); }
void *nedcalloc (size_t no, size_t size) { return calloc (no, size); }
void *nedrealloc (void *mem, size_t size) { return realloc (mem, size); }
void nedfree (void *mem) { free (mem); }
