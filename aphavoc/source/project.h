

































































#define INTERNAL_MODULES 1

#ifndef INTERNAL_MODULES
#define NO_INTERNAL_MODULES 1
#endif





//
// For the small oem demonstration version for 3dlabs
//

#define OEM_3DLABS_VERSION 0

//
// For the demo version (Remake and delete sound block if changed)
//

#define DEMO_VERSION 		0

//
// Initialises 3D object sub-object states for recognition guide
//

#define RECOGNITION_GUIDE 	0





#include "system.h"

#include "graphics.h"

#include "maths.h"

#include "misc.h"

#include "multi.h"

#include "userint2.h"

#include "2d.h"

#include "3d.h"

#include "sound.h"

#ifdef OGRE_EE
// FIXME
#define screen void
#define display_device void

#define read_joystick_values(x)
#define play_ffb_weapon_effect(x, y)

#define create_user_texture_screen(A, B, C, D) NULL
#define create_system_texture_screen(A, B, C, D) NULL
#define create_user_3dvisual_texture_screen(A, B, C) NULL
#define destroy_screen(X)
#define lock_screen(X) 1
#define unlock_screen(X)
#define get_screen_of_system_texture(X) NULL
#define set_block(A, B, C, D, E)
#endif





#include "ai_extrn.h"

#include "cmndline.h"

#include "deinit.h"

#include "flight.h"

#include "game.h"

#include "gfxfiles.h"

#include "global.h"

#include "init.h"

#include "test.h"

#include "update.h"

#include "install.h"

#include "cdmusic.h"





#include "appsound/appsound.h"

#include "comms/comms.h"

#include "entity/entity.h"

#include "ai/ai.h"

#include "app3d/app3d.h"

#include "events/events.h"

#include "gameflow/gameflow.h"

#include "gunships/gunships.h"

#include "misc/miscell.h"

#include "ui_menu/ui_menu.h"

#include "eechini.h"  //VJ 030403

#include "servlog.h" // Jabberwock 031119 Server log

#include "wutcfg.h"   //VJ 030326

#include "gwutcfg.h"   //VJ 040220

#include "sharedmem.h"	// Retro 8Mar2005





extern int
	cdrom_drive_letter;

extern int
	apache_havoc_installation_path_valid;

extern char
	apache_havoc_installation_path[];

// JB 030311 Enable running out of separate directories
extern char
	comanche_hokum_installation_path[];





extern void set_registry_language_settings ( void );




