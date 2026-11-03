

































































#include "project.h"









static const char
	null_text_string[] = "NULL";
/*
static game_text_translation_type
	text_translation_list [] =
	{
		{
			WEAPON_DESCRIPTION_UNKNOWN_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"?",				 			0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"?",			 				0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"?",				 			0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"?",				 			0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"?",				 			0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_HEDP_ROUNDS_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"High explosive dual purpose rounds", 		0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Hochexplosive Mehrzweck-Geschosse", 		0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"High explosive dual purpose rounds", 		0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"High explosive dual purpose rounds", 		0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cartuchos de doble propósito con alto explosivo", 		0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_TWO_BARREL_CANNON_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Two-barreled cannon",				 			0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Doppelläufige Kanone",			 				0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Two-barreled cannon",				 			0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Two-barreled cannon",				 			0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cañón de doble tubo",				 			0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_HE_ROUNDS_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"High explosive rounds",	 					0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Hochexplosive Geschosse",		 				0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"High explosive rounds",	 					0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"High explosive rounds",	 					0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cartuchos de alto explosivo",	 			0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_AP_ROUNDS_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Armour piercing rounds",						0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Panzerbrechende Geschosse",					0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Armour piercing rounds",						0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Armour piercing rounds",						0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cartuchos de metralla",						0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_70MM_HE_ROCKET_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"70mm high explosive unguided rocket",					0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Ungelenkte 70-mm-Rakete mit Brisanzsprengstoff",	0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"70mm high explosive unguided rocket",					0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"70mm high explosive unguided rocket",					0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cohetes de 70 mm, sin guía y con alto explosivo",	0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_70MM_MP_ROCKET_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"70mm multi-purpose unguided rocket",		0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Ungelenkte 70-mm-Mehrzweckrakete",		  	0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"70mm multi-purpose unguided rocket",		0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"70mm multi-purpose unguided rocket",	 	0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cohetes de 70 mm, sin guía y multi-propósito",		0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_80MM_ROCKET_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"80mm unguided rocket",				 			0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Ungelenkte 80-mm-Rakete",				 		0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"80mm unguided rocket",				 			0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"80mm unguided rocket",				 			0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cohetes de 80 mm sin guía",					0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_130MM_ROCKET_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"130mm unguided rocket",  	 					0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Ungelenkte 130-mm-Rakete",  	 				0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"130mm unguided rocket",  	 					0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"130mm unguided rocket",  	 					0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Cohetes de 130 mm sin guía",  				0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_IR_MISSILE_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Infra-red guided air-to-air missile",		0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Infrarotgelenkte Luftkampfrakete",			0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Infra-red guided air-to-air missile",		0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Infra-red guided air-to-air missile",		0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Misil aire-aire guiado por infrarrojos",	0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_RADAR_HELLFIRE_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Radar guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Radargelenkte Panzerabwehr-Rakete",		0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Radar guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Radar guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Misil anti-carro guiado por radar",		0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_LASER_HELLFIRE_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Laser guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Lasergelenkte Panzerabwehr-Rakete",		0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Laser guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Laser guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Misil anti-carro guiado por láser",	 	0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_ATAKA_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Radio command guided anti-tank missile", 	0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Über Funk gesteuerte Panzerabwehr-Rakete",  0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Radio command guided anti-tank missile", 	0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Radio command guided anti-tank missile", 	0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Misil anti-carro guiado por radio", 			0, 0,		0, 0 },
			}
		},

		{
			WEAPON_DESCRIPTION_VIKHR_TEXT,
			{
				{ LANGUAGE_ENGLISH, 	"Laser guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_GERMAN, 	"Lasergelenkte Panzerabwehr-Rakete",		0, 0,		0, 0 },
				{ LANGUAGE_FRENCH, 	"Laser guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_ITALIAN, 	"Laser guided anti-tank missile",		 	0, 0,		0, 0 },
				{ LANGUAGE_SPANISH, 	"Misil anti-carro guiado por láser",	 	0, 0,		0, 0 },
			}
		},
	};
	*/





const char *get_weapon_description_text (int id)
{
	ASSERT ((id >= 0) && (id < NUM_WEAPON_DESCRIPTION_TEXT_ITEMS));

	return null_text_string;	//get_localisation_text (text_translation_list, id);
}




